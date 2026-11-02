//! Persistent ARC state and four-mask representation for the disk profile.
use crate::disk_trie::{DiskAddress, DiskFault, DiskMachine, DiskOutcome, DiskTrie, install_disk};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct DiskArcState {
    pub address: DiskAddress,
    pub height: usize,
    pub width: usize,
}

pub struct DiskArcMask {
    pub address: DiskAddress,
    pub individual_position: DiskAddress,
    pub neighbourhood_position: DiskAddress,
    pub individual_color: DiskAddress,
    pub neighbourhood_color: DiskAddress,
}

pub struct DiskArc {
    trie: DiskTrie,
    arc_root: DiskAddress,
    root: DiskAddress,
    puzzle: Option<Vec<u8>>,
    path: PathBuf,
}

impl DiskArc {
    pub fn create(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let path = path.as_ref().to_path_buf();
        let trie = DiskTrie::create(&path)?;
        let root = trie.root().provision(b"arc")?;
        Ok(Self {
            trie,
            arc_root: root.clone(),
            root,
            puzzle: None,
            path,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, DiskFault> {
        let path = path.as_ref().to_path_buf();
        let trie = DiskTrie::open(&path)?;
        let root = trie.root().child(b"arc")?.ok_or(DiskFault::Corrupt)?;
        Ok(Self {
            trie,
            arc_root: root.clone(),
            root,
            puzzle: None,
            path,
        })
    }

    /// Provision one information-preserving puzzle namespace in a shared persistent trie.
    pub fn provision_puzzle(path: impl AsRef<Path>, puzzle: &[u8]) -> Result<Self, DiskFault> {
        if puzzle.is_empty() {
            return Err(DiskFault::Corrupt);
        }
        let path = path.as_ref().to_path_buf();
        let trie = DiskTrie::create(&path)?;
        let arc_root = trie.root().provision(b"arc")?;
        let root = arc_root.provision(b"puzzles")?.provision(puzzle)?;
        root.provision(b"identity")?
            .provision(puzzle)?
            .terminate()?;
        Ok(Self {
            trie,
            arc_root,
            root,
            puzzle: Some(puzzle.to_vec()),
            path,
        })
    }

    pub fn open_puzzle(path: impl AsRef<Path>, puzzle: &[u8]) -> Result<Self, DiskFault> {
        let path = path.as_ref().to_path_buf();
        let trie = DiskTrie::open(&path)?;
        let arc_root = trie.root().child(b"arc")?.ok_or(DiskFault::Corrupt)?;
        let root = arc_root
            .locate([b"puzzles".as_slice(), puzzle])?
            .ok_or(DiskFault::Corrupt)?;
        Ok(Self {
            trie,
            arc_root,
            root,
            puzzle: Some(puzzle.to_vec()),
            path,
        })
    }

    pub fn close_puzzle(&self) -> Result<(), DiskFault> {
        if self.puzzle.is_none() || self.root.child(b"complete")?.is_some() {
            return Err(DiskFault::AlreadyComplete);
        }
        self.root.provision(b"complete")?.terminate()
    }

    fn ensure_open(&self) -> Result<(), DiskFault> {
        if self.root.child(b"complete")?.is_some() {
            Err(DiskFault::AlreadyComplete)
        } else {
            Ok(())
        }
    }

    pub fn store_root(&self) -> DiskAddress {
        self.trie.root()
    }

    pub fn verify_canonical_traces(&self) -> Result<usize, DiskFault> {
        let traces = self.path.join("arc-canonical-traces");
        let entries = match fs::read_dir(traces) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => return Err(DiskFault::Io),
        };
        let mut verified = 0;
        for entry in entries {
            let entry = entry.map_err(|_| DiskFault::Io)?;
            let bytes = fs::read(entry.path()).map_err(|_| DiskFault::Io)?;
            if bytes.len() < 16 {
                return Err(DiskFault::Corrupt);
            }
            let kind_len =
                u64::from_be_bytes(bytes[..8].try_into().map_err(|_| DiskFault::Corrupt)?) as usize;
            let payload_length_offset = 8usize.checked_add(kind_len).ok_or(DiskFault::Corrupt)?;
            let payload_offset = payload_length_offset
                .checked_add(8)
                .ok_or(DiskFault::Corrupt)?;
            if payload_offset > bytes.len() {
                return Err(DiskFault::Corrupt);
            }
            let payload_len = u64::from_be_bytes(
                bytes[payload_length_offset..payload_offset]
                    .try_into()
                    .map_err(|_| DiskFault::Corrupt)?,
            ) as usize;
            if payload_offset.checked_add(payload_len) != Some(bytes.len()) {
                return Err(DiskFault::Corrupt);
            }
            let digest: [u8; 32] = Sha256::digest(&bytes).into();
            let expected_name = format!("{}.trace", hex_bytes(&digest));
            if entry.file_name().as_encoded_bytes() != expected_name.as_bytes() {
                return Err(DiskFault::Corrupt);
            }
            let kind = &bytes[8..payload_length_offset];
            let address = self
                .arc_root
                .locate([b"content-addressed".as_slice(), kind, digest.as_slice()])?
                .ok_or(DiskFault::Corrupt)?;
            if !address.is_terminal()? {
                return Err(DiskFault::Corrupt);
            }
            verified += 1;
        }
        Ok(verified)
    }

    fn content_addressed(&self, kind: &[u8], payload: &[u8]) -> Result<DiskAddress, DiskFault> {
        let mut canonical = Vec::with_capacity(16 + kind.len() + payload.len());
        canonical.extend_from_slice(&(kind.len() as u64).to_be_bytes());
        canonical.extend_from_slice(kind);
        canonical.extend_from_slice(&(payload.len() as u64).to_be_bytes());
        canonical.extend_from_slice(payload);
        let digest: [u8; 32] = Sha256::digest(&canonical).into();
        let traces = self.path.join("arc-canonical-traces");
        fs::create_dir_all(&traces).map_err(|_| DiskFault::Io)?;
        let trace_path = traces.join(format!("{}.trace", hex_bytes(&digest)));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&trace_path)
        {
            Ok(mut file) => {
                file.write_all(&canonical).map_err(|_| DiskFault::Io)?;
                file.sync_data().map_err(|_| DiskFault::Io)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if fs::read(&trace_path).map_err(|_| DiskFault::Io)? != canonical {
                    return Err(DiskFault::Corrupt);
                }
            }
            Err(_) => return Err(DiskFault::Io),
        }
        let address = self
            .arc_root
            .provision(b"content-addressed")?
            .provision(kind)?
            .provision(&digest)?;
        address.terminate()?;
        Ok(address)
    }

    pub fn encode(
        &self,
        namespace: &[u8],
        episode: &[u8],
        grid: &[Vec<u8>],
        evidence: &DiskAddress,
    ) -> Result<DiskArcState, DiskFault> {
        self.ensure_open()?;
        if namespace != b"input" && namespace != b"output" {
            return Err(DiskFault::Corrupt);
        }
        let height = grid.len();
        let width = grid.first().map_or(0, Vec::len);
        if height == 0 || width == 0 || grid.iter().any(|row| row.len() != width) {
            return Err(DiskFault::Corrupt);
        }
        let state = self
            .root
            .provision(b"states")?
            .provision(namespace)?
            .provision(episode)?;
        state.provision(b"evidence")?.point_to(evidence)?;
        state
            .provision(b"height")?
            .provision(&(height as u64).to_be_bytes())?
            .terminate()?;
        state
            .provision(b"width")?
            .provision(&(width as u64).to_be_bytes())?
            .terminate()?;
        let mut payload = b"disk-arc-state-v1".to_vec();
        payload.extend_from_slice(&(namespace.len() as u64).to_be_bytes());
        payload.extend_from_slice(namespace);
        payload.extend_from_slice(&(height as u64).to_be_bytes());
        payload.extend_from_slice(&(width as u64).to_be_bytes());
        payload.extend(grid.iter().flatten().copied());
        let canonical = self.content_addressed(b"arc-state", &payload)?;
        state.provision(b"canonical-state")?.point_to(&canonical)?;
        Ok(DiskArcState {
            address: state,
            height,
            width,
        })
    }

    /// Materialize a predicted ARC grid through persistent source/target/continuation copy
    /// instructions. The returned bytes are decoded from the disk output cells, rather than
    /// returned from the caller's prediction buffer.
    pub fn execute_prediction(
        &self,
        episode: &[u8],
        prediction: &[Vec<u8>],
        evidence: &DiskAddress,
    ) -> Result<Vec<Vec<u8>>, DiskFault> {
        self.ensure_open()?;
        let height = prediction.len();
        let width = prediction.first().map_or(0, Vec::len);
        if height == 0 || width == 0 || prediction.iter().any(|row| row.len() != width) {
            return Err(DiskFault::Corrupt);
        }
        let executions = self.root.provision(b"executions")?;
        if let Some(existing) = executions.child(episode)? {
            return if existing.child(b"complete")?.is_some() {
                Err(DiskFault::AlreadyComplete)
            } else {
                Err(DiskFault::Partial)
            };
        }
        let run = executions.provision(episode)?;
        run.provision(b"evidence")?.point_to(evidence)?;
        let mut machine = DiskMachine::create(&self.path)?;
        run.provision(b"height")?
            .provision(&(height as u64).to_be_bytes())?
            .terminate()?;
        run.provision(b"width")?
            .provision(&(width as u64).to_be_bytes())?
            .terminate()?;
        let values = self.root.provision(b"literal-colors")?;
        let program = self.root.provision(b"cell-copy-program")?;
        let source_slot = program.provision(b"source-slot")?;
        let next_slot = program.provision(b"next-slot")?;
        let effect_slot = program.provision(b"effect-slot")?;
        let instruction = program.provision(b"instruction")?;
        let halt = program.provision(b"halt")?;
        halt.terminate()?;
        next_slot.point_to(&halt)?;
        install_disk(&instruction, &source_slot, &effect_slot, &next_slot)?;
        let traces = self.path.join("arc-execution-traces");
        fs::create_dir_all(&traces).map_err(|_| DiskFault::Io)?;
        let mut trace_identity = Vec::new();
        if let Some(puzzle) = &self.puzzle {
            trace_identity.extend_from_slice(&(puzzle.len() as u64).to_be_bytes());
            trace_identity.extend_from_slice(puzzle);
        }
        trace_identity.extend_from_slice(episode);
        let trace_name = trace_name(&trace_identity);
        let trace_path = traces.join(&trace_name);
        let mut trace = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&trace_path)
            .map_err(|_| DiskFault::Io)?;
        let mut actual_cycles = 0u64;
        for (index, color) in prediction.iter().flatten().copied().enumerate() {
            let value = values.provision(&[color])?;
            value.terminate()?;
            source_slot.point_to(&value)?;
            effect_slot.unset()?;
            machine.enter(&instruction)?;
            if machine.cycle()? != DiskOutcome::Advanced {
                return Err(DiskFault::Corrupt);
            }
            actual_cycles += 1;
            let copied = effect_slot.follow()?;
            if !copied.same(&value) {
                return Err(DiskFault::Corrupt);
            }
            trace
                .write_all(&(index as u64).to_be_bytes())
                .and_then(|_| trace.write_all(&[color]))
                .and_then(|_| trace.write_all(&value.identity_key()))
                .and_then(|_| trace.write_all(&instruction.identity_key()))
                .map_err(|_| DiskFault::Io)?;
        }
        if machine.cycle()? != DiskOutcome::Halted {
            return Err(DiskFault::Corrupt);
        }
        trace.sync_data().map_err(|_| DiskFault::Io)?;
        drop(trace);
        let mut bytes = Vec::new();
        fs::File::open(&trace_path)
            .and_then(|mut file| file.read_to_end(&mut bytes))
            .map_err(|_| DiskFault::Io)?;
        const RECORD_BYTES: usize = 25;
        if bytes.len() != height * width * RECORD_BYTES {
            return Err(DiskFault::Corrupt);
        }
        let mut output = vec![vec![0; width]; height];
        for record in bytes.chunks_exact(RECORD_BYTES) {
            let index = u64::from_be_bytes(record[..8].try_into().map_err(|_| DiskFault::Corrupt)?)
                as usize;
            if index >= height * width {
                return Err(DiskFault::Corrupt);
            }
            let source_key: [u8; 8] = record[9..17].try_into().map_err(|_| DiskFault::Corrupt)?;
            let instruction_key: [u8; 8] =
                record[17..25].try_into().map_err(|_| DiskFault::Corrupt)?;
            if source_key != values.provision(&[record[8]])?.identity_key()
                || instruction_key != instruction.identity_key()
            {
                return Err(DiskFault::Corrupt);
            }
            output[index / width][index % width] = record[8];
        }
        run.provision(b"output-trace")?
            .provision(trace_name.as_bytes())?
            .terminate()?;
        run.provision(b"trace-record-bytes")?
            .provision(&(RECORD_BYTES as u64).to_be_bytes())?
            .terminate()?;
        run.provision(b"actual-cycles")?
            .provision(&actual_cycles.to_be_bytes())?
            .terminate()?;
        run.provision(b"complete")?.terminate()?;
        Ok(output)
    }

    pub fn masks_equal(
        &self,
        expected: &DiskArcMask,
        actual: &DiskArcMask,
    ) -> Result<bool, DiskFault> {
        Ok([
            (&expected.individual_position, &actual.individual_position),
            (
                &expected.neighbourhood_position,
                &actual.neighbourhood_position,
            ),
            (&expected.individual_color, &actual.individual_color),
            (&expected.neighbourhood_color, &actual.neighbourhood_color),
        ]
        .into_iter()
        .all(|(left, right)| left.same(right)))
    }

    fn canonical_mask_component(
        &self,
        kind: &[u8],
        mut events: Vec<Vec<Vec<u8>>>,
    ) -> Result<DiskAddress, DiskFault> {
        events.sort();
        let mut payload = b"disk-arc-mask-component-v1".to_vec();
        payload.extend_from_slice(&(kind.len() as u64).to_be_bytes());
        payload.extend_from_slice(kind);
        payload.extend_from_slice(&(events.len() as u64).to_be_bytes());
        for event in events {
            payload.extend_from_slice(&(event.len() as u64).to_be_bytes());
            for token in event {
                payload.extend_from_slice(&(token.len() as u64).to_be_bytes());
                payload.extend_from_slice(&token);
            }
        }
        self.content_addressed(b"arc-mask-component", &payload)
    }

    pub fn derive_masks(
        &self,
        episode: &[u8],
        input: &[Vec<u8>],
        output: &[Vec<u8>],
        evidence: &DiskAddress,
    ) -> Result<DiskArcMask, DiskFault> {
        self.ensure_open()?;
        let height = input.len();
        let width = input.first().map_or(0, Vec::len);
        if height == 0
            || width == 0
            || output.len() != height
            || input.iter().chain(output).any(|row| row.len() != width)
        {
            return Err(DiskFault::Corrupt);
        }
        let mask = self.root.provision(b"masks")?.provision(episode)?;
        mask.provision(b"evidence")?.point_to(evidence)?;
        let mut individual_position_events = Vec::new();
        let mut neighbourhood_position_events = Vec::new();
        let mut individual_color_events = Vec::new();
        let mut neighbourhood_color_events = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let position_changed = (input[y][x] == 0) != (output[y][x] == 0);
                let color_changed = input[y][x] != output[y][x];
                if position_changed {
                    individual_position_events.push(change_event(y, x, input[y][x], output[y][x]));
                }
                if color_changed {
                    individual_color_events.push(change_event(y, x, input[y][x], output[y][x]));
                }
                for (port, dy, dx) in directions() {
                    let Some(ny) = y.checked_add_signed(dy) else {
                        continue;
                    };
                    let Some(nx) = x.checked_add_signed(dx) else {
                        continue;
                    };
                    if ny >= height || nx >= width {
                        continue;
                    }
                    let neighbour_position_changed = (input[ny][nx] == 0) != (output[ny][nx] == 0);
                    let neighbour_color_changed = input[ny][nx] != output[ny][nx];
                    if neighbour_position_changed {
                        neighbourhood_position_events.push(neighbour_change_event(
                            y,
                            x,
                            port,
                            input[ny][nx],
                            output[ny][nx],
                        ));
                    }
                    if neighbour_color_changed {
                        neighbourhood_color_events.push(neighbour_change_event(
                            y,
                            x,
                            port,
                            input[ny][nx],
                            output[ny][nx],
                        ));
                    }
                }
            }
        }
        let individual_position =
            self.canonical_mask_component(b"individual-position", individual_position_events)?;
        let neighbourhood_position = self
            .canonical_mask_component(b"neighbourhood-position", neighbourhood_position_events)?;
        let individual_color =
            self.canonical_mask_component(b"individual-color", individual_color_events)?;
        let neighbourhood_color =
            self.canonical_mask_component(b"neighbourhood-color", neighbourhood_color_events)?;
        for (kind, component) in [
            (b"individual-position".as_slice(), &individual_position),
            (b"neighbourhood-position", &neighbourhood_position),
            (b"individual-color", &individual_color),
            (b"neighbourhood-color", &neighbourhood_color),
        ] {
            mask.provision(kind)?.point_to(component)?;
        }
        Ok(DiskArcMask {
            address: mask,
            individual_position,
            neighbourhood_position,
            individual_color,
            neighbourhood_color,
        })
    }
}

fn change_event(y: usize, x: usize, before: u8, after: u8) -> Vec<Vec<u8>> {
    vec![
        (y as u64).to_be_bytes().to_vec(),
        (x as u64).to_be_bytes().to_vec(),
        vec![before],
        vec![after],
    ]
}

fn neighbour_change_event(y: usize, x: usize, port: &[u8], before: u8, after: u8) -> Vec<Vec<u8>> {
    vec![
        (y as u64).to_be_bytes().to_vec(),
        (x as u64).to_be_bytes().to_vec(),
        port.to_vec(),
        vec![before],
        vec![after],
    ]
}

fn trace_name(episode: &[u8]) -> String {
    format!("{}.trace", hex_bytes(episode))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 15) as usize] as char);
    }
    encoded
}

fn directions() -> [(&'static [u8], isize, isize); 8] {
    [
        (b"north-west", -1, -1),
        (b"north", -1, 0),
        (b"north-east", -1, 1),
        (b"west", 0, -1),
        (b"east", 0, 1),
        (b"south-west", 1, -1),
        (b"south", 1, 0),
        (b"south-east", 1, 1),
    ]
}
