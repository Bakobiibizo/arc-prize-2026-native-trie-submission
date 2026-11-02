//! Paired graph episodes under literal `input` and `output` trie addresses.
//! Ordered occurrence traces are disk-backed; only canonical shallow event paths stay resident.
use crate::{Address, Fault, Trie, axis_context::AxisEvent};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static TRACE_SESSION: AtomicU64 = AtomicU64::new(0);
const KEY_BYTES: usize = std::mem::size_of::<usize>();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    Empty,
    Shape,
    Missing,
    Corrupt,
    Reference(Fault),
    Io,
}
impl From<Fault> for StateError {
    fn from(value: Fault) -> Self {
        Self::Reference(value)
    }
}

#[derive(Clone)]
pub struct EncodedState {
    pub namespace: Address,
    pub endpoint: Address,
    pub evidence: Address,
    trace: PathBuf,
    events: usize,
}
#[derive(Clone)]
pub struct AxisTraits {
    pub position: Address,
    pub neighbour_position: Address,
    pub color: Address,
    pub neighbour_color: Address,
}
#[derive(Clone)]
pub struct AxisDelta {
    pub input: AxisTraits,
    pub output: AxisTraits,
    pub position_changed: bool,
    pub neighbour_position_changed: bool,
    pub color_changed: bool,
    pub neighbour_color_changed: bool,
}

pub struct PairedStates {
    storage: Trie,
    trace_root: PathBuf,
    serial: u64,
}
impl Default for PairedStates {
    fn default() -> Self {
        let session = TRACE_SESSION.fetch_add(1, Ordering::Relaxed);
        let trace_root = std::env::var_os("NATIVE_TRIE_TRACE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("native-trie-traces"))
            .join("states")
            .join(format!("{}-{session}", std::process::id()));
        fs::create_dir_all(&trace_root).expect("create state trace directory");
        Self {
            storage: Trie::default(),
            trace_root,
            serial: 0,
        }
    }
}
impl PairedStates {
    pub fn root(&self) -> Address {
        self.storage.root()
    }
    pub fn trace_root(&self) -> &std::path::Path {
        &self.trace_root
    }

    fn edge(parent: &Address, axis_name: &[u8], value: &Address) -> Result<Address, StateError> {
        let axis = parent.provision(axis_name);
        let key = value.identity_key();
        if let Some(at) = axis.child(&key) {
            if !at
                .child(b"trait")
                .ok_or(StateError::Corrupt)?
                .follow()?
                .same(value)
                || at.child(b"axis").and_then(|a| a.child(axis_name)).is_none()
            {
                return Err(StateError::Corrupt);
            }
            return Ok(at);
        }
        let at = axis.provision(&key);
        at.provision(b"trait").point_to(value);
        at.provision(b"axis").provision(axis_name).terminate();
        Ok(at)
    }

    fn recover(namespace: &Address, keys: &[[u8; KEY_BYTES]; 4]) -> Result<AxisTraits, StateError> {
        let mut at = namespace.clone();
        let mut traits = Vec::with_capacity(4);
        for (axis, key) in [
            b"position".as_slice(),
            b"neighbour-position".as_slice(),
            b"color".as_slice(),
            b"neighbour-color".as_slice(),
        ]
        .into_iter()
        .zip(keys)
        {
            at = at
                .child(axis)
                .ok_or(StateError::Missing)?
                .child(key)
                .ok_or(StateError::Missing)?;
            traits.push(at.child(b"trait").ok_or(StateError::Corrupt)?.follow()?);
        }
        Ok(AxisTraits {
            position: traits[0].clone(),
            neighbour_position: traits[1].clone(),
            color: traits[2].clone(),
            neighbour_color: traits[3].clone(),
        })
    }

    /// Canonical event paths stay in the trie. The potentially large ordered graph occurrence is
    /// an append-only fixed-width disk trace of those native path keys.
    pub fn encode(
        &mut self,
        namespace: &[u8],
        graph: &[AxisEvent],
        evidence: &Address,
    ) -> Result<EncodedState, StateError> {
        if graph.is_empty() {
            return Err(StateError::Empty);
        }
        if namespace != b"input" && namespace != b"output" {
            return Err(StateError::Corrupt);
        }
        let root = self.root().provision(namespace);
        let endpoint = self
            .root()
            .provision(b"state-episodes")
            .provision(&evidence.identity_key())
            .provision(namespace);
        endpoint.provision(b"evidence").point_to(evidence);
        let trace = self.trace_root.join(format!("{:016x}.trace", self.serial));
        self.serial += 1;
        let file = File::create(&trace).map_err(|_| StateError::Io)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(b"NTS1").map_err(|_| StateError::Io)?;
        writer
            .write_all(&(graph.len() as u64).to_be_bytes())
            .map_err(|_| StateError::Io)?;
        for event in graph {
            let mut at = root.clone();
            for (axis, value) in [
                (b"position".as_slice(), &event.position),
                (b"neighbour-position".as_slice(), &event.neighbour_position),
                (b"color".as_slice(), &event.color),
                (b"neighbour-color".as_slice(), &event.neighbour_color),
            ] {
                at = Self::edge(&at, axis, value)?;
                writer
                    .write_all(&value.identity_key())
                    .map_err(|_| StateError::Io)?;
            }
        }
        writer.flush().map_err(|_| StateError::Io)?;
        endpoint
            .provision(b"trace-file")
            .provision(trace.as_os_str().as_encoded_bytes())
            .terminate();
        Ok(EncodedState {
            namespace: root,
            endpoint,
            evidence: evidence.clone(),
            trace,
            events: graph.len(),
        })
    }

    fn decode(state: &EncodedState) -> Result<Vec<AxisTraits>, StateError> {
        let mut reader = BufReader::new(File::open(&state.trace).map_err(|_| StateError::Io)?);
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).map_err(|_| StateError::Io)?;
        if &magic != b"NTS1" {
            return Err(StateError::Corrupt);
        }
        let mut count = [0; 8];
        reader.read_exact(&mut count).map_err(|_| StateError::Io)?;
        if u64::from_be_bytes(count) as usize != state.events {
            return Err(StateError::Corrupt);
        }
        let mut result = Vec::with_capacity(state.events);
        for _ in 0..state.events {
            let mut keys = [[0; KEY_BYTES]; 4];
            for key in &mut keys {
                reader.read_exact(key).map_err(|_| StateError::Io)?;
            }
            result.push(Self::recover(&state.namespace, &keys)?);
        }
        let mut trailing = [0];
        if reader.read(&mut trailing).map_err(|_| StateError::Io)? != 0 {
            return Err(StateError::Corrupt);
        }
        Ok(result)
    }

    pub fn compare(
        input: &EncodedState,
        output: &EncodedState,
    ) -> Result<Vec<AxisDelta>, StateError> {
        let input = Self::decode(input)?;
        let output = Self::decode(output)?;
        if input.len() != output.len() {
            return Err(StateError::Shape);
        }
        Ok(input
            .into_iter()
            .zip(output)
            .map(|(before, after)| AxisDelta {
                position_changed: !before.position.same(&after.position),
                neighbour_position_changed: !before
                    .neighbour_position
                    .same(&after.neighbour_position),
                color_changed: !before.color.same(&after.color),
                neighbour_color_changed: !before.neighbour_color.same(&after.neighbour_color),
                input: before,
                output: after,
            })
            .collect())
    }
}
