//! Disk-resident trie profile substrate.
//!
//! Each handle retains only a store reference and node id. Tokens, child edges, and content live
//! in append-only/node-local files and are read for traversal. This is the migration target for
//! the older `Rc<Node>` experimental trie; it does not claim that migration is complete.
use std::{
    cell::Cell,
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskFault {
    Io,
    Corrupt,
    Unset,
    Partial,
    AlreadyComplete,
    TerminalUsedAsReference,
}

struct Store {
    root: PathBuf,
    next_id: Cell<u64>,
    reserved_end: Cell<u64>,
}

#[derive(Clone)]
pub struct DiskAddress {
    store: Rc<Store>,
    id: u64,
}

impl DiskAddress {
    fn node_dir(&self) -> PathBuf {
        self.store
            .root
            .join("nodes")
            .join(format!("{:016x}", self.id))
    }

    pub fn same(&self, other: &Self) -> bool {
        self.id == other.id && self.store.root == other.store.root
    }

    pub fn identity_key(&self) -> [u8; 8] {
        self.id.to_be_bytes()
    }

    pub fn token(&self) -> Result<Vec<u8>, DiskFault> {
        fs::read(self.node_dir().join("token")).map_err(|_| DiskFault::Io)
    }

    pub fn child(&self, token: &[u8]) -> Result<Option<Self>, DiskFault> {
        let edge = self.node_dir().join("edges").join(hex(token));
        let bytes = match fs::read(edge) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(DiskFault::Io),
        };
        let id = decode_id(&bytes)?;
        Ok(Some(Self {
            store: self.store.clone(),
            id,
        }))
    }

    pub fn children(&self) -> Result<Vec<Self>, DiskFault> {
        let mut children = Vec::new();
        for entry in fs::read_dir(self.node_dir().join("edges")).map_err(|_| DiskFault::Io)? {
            let bytes =
                fs::read(entry.map_err(|_| DiskFault::Io)?.path()).map_err(|_| DiskFault::Io)?;
            children.push(Self {
                store: self.store.clone(),
                id: decode_id(&bytes)?,
            });
        }
        children.sort_by_key(Self::identity_key);
        Ok(children)
    }

    pub fn locate<'a>(
        &self,
        tokens: impl IntoIterator<Item = &'a [u8]>,
    ) -> Result<Option<Self>, DiskFault> {
        let mut at = self.clone();
        for token in tokens {
            let Some(next) = at.child(token)? else {
                return Ok(None);
            };
            at = next;
        }
        Ok(Some(at))
    }

    pub fn provision(&self, token: &[u8]) -> Result<Self, DiskFault> {
        if let Some(found) = self.child(token)? {
            return Ok(found);
        }
        let id = allocate_id(&self.store)?;
        let node = Self {
            store: self.store.clone(),
            id,
        };
        fs::create_dir_all(node.node_dir().join("edges")).map_err(|_| DiskFault::Io)?;
        atomic_write(&node.node_dir().join("token"), token)?;
        atomic_write(&node.node_dir().join("content"), b"U")?;
        let edge = self.node_dir().join("edges").join(hex(token));
        match OpenOptions::new().write(true).create_new(true).open(&edge) {
            Ok(mut file) => file
                .write_all(&id.to_be_bytes())
                .map_err(|_| DiskFault::Io)?,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                fs::remove_dir_all(node.node_dir()).map_err(|_| DiskFault::Io)?;
                return self.child(token)?.ok_or(DiskFault::Corrupt);
            }
            Err(_) => return Err(DiskFault::Io),
        }
        Ok(node)
    }

    pub fn unset(&self) -> Result<(), DiskFault> {
        atomic_write(&self.node_dir().join("content"), b"U")
    }

    pub fn terminate(&self) -> Result<(), DiskFault> {
        atomic_write(&self.node_dir().join("content"), b"E")
    }

    pub fn is_terminal(&self) -> Result<bool, DiskFault> {
        Ok(fs::read(self.node_dir().join("content")).map_err(|_| DiskFault::Io)? == b"E")
    }

    pub fn value(&self) -> Result<DiskValue, DiskFault> {
        let bytes = fs::read(self.node_dir().join("content")).map_err(|_| DiskFault::Io)?;
        match bytes.first() {
            Some(b'U') if bytes.len() == 1 => Ok(DiskValue::Unset),
            Some(b'E') if bytes.len() == 1 => Ok(DiskValue::End),
            Some(b'L') if bytes.len() == 9 => {
                Ok(DiskValue::Link(decode_id(&bytes[1..])?.to_be_bytes()))
            }
            _ => Err(DiskFault::Corrupt),
        }
    }

    fn snapshot(&self) -> Result<DiskContent, DiskFault> {
        let bytes = fs::read(self.node_dir().join("content")).map_err(|_| DiskFault::Io)?;
        match bytes.first() {
            Some(b'E') if bytes.len() == 1 => Ok(DiskContent::End),
            Some(b'L') if bytes.len() == 9 => Ok(DiskContent::Link(decode_id(&bytes[1..])?)),
            Some(b'U') if bytes.len() == 1 => Err(DiskFault::Unset),
            _ => Err(DiskFault::Corrupt),
        }
    }

    fn place(&self, content: DiskContent) -> Result<(), DiskFault> {
        match content {
            DiskContent::End => self.terminate(),
            DiskContent::Link(id) => {
                let mut value = Vec::with_capacity(9);
                value.push(b'L');
                value.extend_from_slice(&id.to_be_bytes());
                atomic_write(&self.node_dir().join("content"), &value)
            }
        }
    }

    pub fn point_to(&self, target: &Self) -> Result<(), DiskFault> {
        if self.store.root != target.store.root {
            return Err(DiskFault::Corrupt);
        }
        let mut value = Vec::with_capacity(9);
        value.push(b'L');
        value.extend_from_slice(&target.id.to_be_bytes());
        atomic_write(&self.node_dir().join("content"), &value)
    }

    pub fn follow(&self) -> Result<Self, DiskFault> {
        let bytes = fs::read(self.node_dir().join("content")).map_err(|_| DiskFault::Io)?;
        match bytes.first() {
            Some(b'U') if bytes.len() == 1 => Err(DiskFault::Unset),
            Some(b'E') if bytes.len() == 1 => Err(DiskFault::TerminalUsedAsReference),
            Some(b'L') if bytes.len() == 9 => Ok(Self {
                store: self.store.clone(),
                id: decode_id(&bytes[1..])?,
            }),
            _ => Err(DiskFault::Corrupt),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskValue {
    Unset,
    End,
    Link([u8; 8]),
}

#[derive(Clone, Copy)]
enum DiskContent {
    End,
    Link(u64),
}

pub struct DiskTrie {
    store: Rc<Store>,
}

impl DiskTrie {
    pub fn create(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let root = path.as_ref().to_path_buf();
        fs::create_dir_all(root.join("nodes/0000000000000000/edges")).map_err(|_| DiskFault::Io)?;
        atomic_write(&root.join("nodes/0000000000000000/token"), b"")?;
        atomic_write(&root.join("nodes/0000000000000000/content"), b"U")?;
        if !root.join("next-id").exists() {
            atomic_write(&root.join("next-id"), &1u64.to_be_bytes())?;
        }
        Ok(Self {
            store: Rc::new(Store {
                root,
                next_id: Cell::new(0),
                reserved_end: Cell::new(0),
            }),
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let root = path.as_ref().to_path_buf();
        if !root.join("nodes/0000000000000000").is_dir() || !root.join("next-id").is_file() {
            return Err(DiskFault::Corrupt);
        }
        Ok(Self {
            store: Rc::new(Store {
                root,
                next_id: Cell::new(0),
                reserved_end: Cell::new(0),
            }),
        })
    }

    pub fn root(&self) -> DiskAddress {
        DiskAddress {
            store: self.store.clone(),
            id: 0,
        }
    }

    pub fn provision(&self, tokens: &[&[u8]]) -> Result<DiskAddress, DiskFault> {
        let mut at = self.root();
        for token in tokens {
            at = at.provision(token)?;
        }
        Ok(at)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskOutcome {
    Advanced,
    Halted,
}

/// Native copy/follow machine over persistent addresses. State, output, slots, pointer, and
/// continuations occupy one disk store under explicit namespaces.
pub struct DiskMachine {
    trie: DiskTrie,
    states: DiskAddress,
    output: DiskAddress,
    pointer: DiskAddress,
    pub cycles: u64,
}

impl DiskMachine {
    pub fn create(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let trie = DiskTrie::create(path)?;
        let root = trie.root();
        let states = root.provision(b"machine-states")?;
        let output = root.provision(b"machine-output")?;
        let pointer = states.provision(b"instruction-pointer")?;
        Ok(Self {
            trie,
            states,
            output,
            pointer,
            cycles: 0,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let trie = DiskTrie::open(path)?;
        let root = trie.root();
        let states = root.child(b"machine-states")?.ok_or(DiskFault::Corrupt)?;
        let output = root.child(b"machine-output")?.ok_or(DiskFault::Corrupt)?;
        let pointer = states
            .child(b"instruction-pointer")?
            .ok_or(DiskFault::Corrupt)?;
        Ok(Self {
            trie,
            states,
            output,
            pointer,
            cycles: 0,
        })
    }

    pub fn root(&self) -> DiskAddress {
        self.trie.root()
    }
    pub fn states(&self) -> DiskAddress {
        self.states.clone()
    }
    pub fn output(&self) -> DiskAddress {
        self.output.clone()
    }
    pub fn pointer(&self) -> DiskAddress {
        self.pointer.clone()
    }

    pub fn enter(&mut self, state: &DiskAddress) -> Result<(), DiskFault> {
        self.pointer.point_to(state)?;
        self.cycles = 0;
        Ok(())
    }

    pub fn current(&self) -> Result<DiskAddress, DiskFault> {
        self.pointer.follow()
    }

    pub fn cycle(&mut self) -> Result<DiskOutcome, DiskFault> {
        let state = self.pointer.follow()?;
        if state.is_terminal()? {
            return Ok(DiskOutcome::Halted);
        }
        let source = state
            .child(b"source")?
            .ok_or(DiskFault::Corrupt)?
            .follow()?;
        let target = state
            .child(b"target")?
            .ok_or(DiskFault::Corrupt)?
            .follow()?;
        let next_cell = state.child(b"next")?.ok_or(DiskFault::Corrupt)?.follow()?;
        let content = source.snapshot()?;
        let next = if target.same(&next_cell) {
            match content {
                DiskContent::Link(id) => DiskAddress {
                    store: state.store.clone(),
                    id,
                },
                DiskContent::End => return Err(DiskFault::TerminalUsedAsReference),
            }
        } else {
            next_cell.follow()?
        };
        target.place(content)?;
        self.pointer.point_to(&next)?;
        self.cycles += 1;
        Ok(DiskOutcome::Advanced)
    }
}

pub struct DiskRetrieval {
    pub endpoint: DiskAddress,
    pub constructions: Vec<DiskAddress>,
    pub evidence: Vec<DiskAddress>,
}

/// Exact episodic retrieval profile. It returns every addressed candidate; selection is not part
/// of retrieval. Structural backoff is intentionally absent until a profile order is declared.
pub struct DiskEpisodes {
    root: DiskAddress,
}

impl DiskEpisodes {
    pub fn new(namespace: &DiskAddress) -> Result<Self, DiskFault> {
        Ok(Self {
            root: namespace.provision(b"episodes")?,
        })
    }

    fn endpoint(&self, cue: &[&[u8]], create: bool) -> Result<Option<DiskAddress>, DiskFault> {
        let mut at = self.root.clone();
        for token in cue {
            at = if create {
                at.provision(token)?
            } else {
                let Some(child) = at.child(token)? else {
                    return Ok(None);
                };
                child
            };
        }
        Ok(Some(at))
    }

    pub fn record(
        &self,
        cue: &[&[u8]],
        construction: Option<&DiskAddress>,
        evidence: &DiskAddress,
    ) -> Result<DiskAddress, DiskFault> {
        let endpoint = self.endpoint(cue, true)?.ok_or(DiskFault::Corrupt)?;
        let evidence_slot = endpoint
            .provision(b"evidence")?
            .provision(&evidence.identity_key())?;
        evidence_slot.point_to(evidence)?;
        if let Some(construction) = construction {
            let candidate = endpoint
                .provision(b"constructions")?
                .provision(&construction.identity_key())?;
            candidate.point_to(construction)?;
        }
        Ok(endpoint)
    }

    pub fn retrieve(&self, cue: &[&[u8]]) -> Result<Option<DiskRetrieval>, DiskFault> {
        let Some(endpoint) = self.endpoint(cue, false)? else {
            return Ok(None);
        };
        let constructions = match endpoint.child(b"constructions")? {
            Some(root) => root
                .children()?
                .into_iter()
                .map(|candidate| candidate.follow())
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        let evidence = match endpoint.child(b"evidence")? {
            Some(root) => root
                .children()?
                .into_iter()
                .map(|item| item.follow())
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        Ok(Some(DiskRetrieval {
            endpoint,
            constructions,
            evidence,
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeAssessment {
    Support,
    Oppose,
    Unclassified,
}

/// Persistent prediction/receipt association. Outcome interpretation is explicit and never
/// inferred from the mere arrival of an outcome token.
pub struct DiskReceipts {
    root: DiskAddress,
}

impl DiskReceipts {
    pub fn new(namespace: &DiskAddress) -> Result<Self, DiskFault> {
        Ok(Self {
            root: namespace.provision(b"receipts")?,
        })
    }

    pub fn issue(
        &self,
        receipt_token: &[u8],
        context: &DiskAddress,
        prediction: &DiskAddress,
        evidence: &[DiskAddress],
        selection_policy: &[u8],
    ) -> Result<DiskAddress, DiskFault> {
        if self.root.child(receipt_token)?.is_some() {
            return Err(DiskFault::Corrupt);
        }
        let receipt = self.root.provision(receipt_token)?;
        receipt.provision(b"context")?.point_to(context)?;
        receipt.provision(b"prediction")?.point_to(prediction)?;
        receipt
            .provision(b"selection-policy")?
            .provision(selection_policy)?
            .terminate()?;
        let evidence_root = receipt.provision(b"evidence")?;
        for item in evidence {
            evidence_root
                .provision(&item.identity_key())?
                .point_to(item)?;
        }
        Ok(receipt)
    }

    pub fn associate_outcome(
        &self,
        receipt_token: &[u8],
        outcome_token: &[u8],
        assessment: OutcomeAssessment,
    ) -> Result<DiskAddress, DiskFault> {
        let receipt = self.root.child(receipt_token)?.ok_or(DiskFault::Corrupt)?;
        let outcome = receipt.provision(b"outcomes")?.provision(outcome_token)?;
        let assessment = match assessment {
            OutcomeAssessment::Support => b"support".as_slice(),
            OutcomeAssessment::Oppose => b"oppose",
            OutcomeAssessment::Unclassified => b"unclassified",
        };
        outcome
            .provision(b"assessment")?
            .provision(assessment)?
            .terminate()?;
        Ok(outcome)
    }

    pub fn receipt(&self, receipt_token: &[u8]) -> Result<Option<DiskAddress>, DiskFault> {
        self.root.child(receipt_token)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskAttemptStatus {
    Completed,
    Limit,
    Fault(DiskFault),
}

pub struct DiskAttempt {
    pub construction: DiskAddress,
    pub before: Option<DiskValue>,
    pub after: Option<DiskValue>,
    pub cycles: u64,
    pub status: DiskAttemptStatus,
}

/// Attempt every exactly retrieved construction. This bounded profile requires each construction
/// to declare an isolated `effect` slot and an `entry`; shared-state commit is deliberately not
/// inferred from host order.
pub fn attempt_all_isolated(
    machine: &mut DiskMachine,
    constructions: &[DiskAddress],
    cycle_limit: usize,
) -> Result<Vec<DiskAttempt>, DiskFault> {
    let mut attempts = Vec::with_capacity(constructions.len());
    for construction in constructions {
        let mut before = None;
        let mut after = None;
        let result = (|| {
            let entry = construction
                .child(b"entry")?
                .ok_or(DiskFault::Corrupt)?
                .follow()?;
            let effect = construction
                .child(b"effect")?
                .ok_or(DiskFault::Corrupt)?
                .follow()?;
            before = Some(effect.value()?);
            machine.enter(&entry)?;
            for _ in 0..cycle_limit {
                if machine.cycle()? == DiskOutcome::Halted {
                    after = Some(effect.value()?);
                    return Ok(DiskAttemptStatus::Completed);
                }
            }
            after = Some(effect.value()?);
            Ok(DiskAttemptStatus::Limit)
        })();
        let status = match result {
            Ok(status) => status,
            Err(fault) => DiskAttemptStatus::Fault(fault),
        };
        attempts.push(DiskAttempt {
            construction: construction.clone(),
            before,
            after,
            cycles: machine.cycles,
            status,
        });
    }
    Ok(attempts)
}

pub fn install_disk(
    at: &DiskAddress,
    source: &DiskAddress,
    target: &DiskAddress,
    next_cell: &DiskAddress,
) -> Result<(), DiskFault> {
    at.provision(b"source")?.point_to(source)?;
    at.provision(b"target")?.point_to(target)?;
    at.provision(b"next")?.point_to(next_cell)
}

fn allocate_id(store: &Store) -> Result<u64, DiskFault> {
    let cached = store.next_id.get();
    if cached < store.reserved_end.get() {
        store.next_id.set(cached + 1);
        return Ok(cached);
    }
    // The 1,024-id reservation is a durability/performance boundary chosen from the 4,000-event
    // baseline: it reduces one synchronous counter write per node to at most four per 4,000
    // allocations. A crash may leave unused ids, but cannot republish them after the reservation
    // counter reaches disk.
    const RESERVATION: u64 = 1_024;
    let path = store.root.join("next-id");
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|_| DiskFault::Io)?;
    let mut bytes = [0; 8];
    file.read_exact(&mut bytes).map_err(|_| DiskFault::Io)?;
    let id = u64::from_be_bytes(bytes);
    let next = id.checked_add(RESERVATION).ok_or(DiskFault::Corrupt)?;
    file.set_len(0).map_err(|_| DiskFault::Io)?;
    file.seek(SeekFrom::Start(0)).map_err(|_| DiskFault::Io)?;
    file.write_all(&next.to_be_bytes())
        .map_err(|_| DiskFault::Io)?;
    file.sync_data().map_err(|_| DiskFault::Io)?;
    store.next_id.set(id + 1);
    store.reserved_end.set(next);
    Ok(id)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), DiskFault> {
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|_| DiskFault::Io)?;
    fs::rename(temporary, path).map_err(|_| DiskFault::Io)
}

fn decode_id(bytes: &[u8]) -> Result<u64, DiskFault> {
    let bytes: [u8; 8] = bytes.try_into().map_err(|_| DiskFault::Corrupt)?;
    Ok(u64::from_be_bytes(bytes))
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 15) as usize] as char);
    }
    encoded
}
