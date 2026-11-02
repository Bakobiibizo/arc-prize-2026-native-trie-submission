//! Trie-represented structural change masks derived from paired state paths.
//! Canonical mask events remain resident; ordered occurrence traces are disk-backed.
use crate::{Address, Fault, Trie, paired_states::AxisDelta};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static MASK_SESSION: AtomicU64 = AtomicU64::new(0);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskError {
    Empty,
    Corrupt,
    Reference(Fault),
    Io,
}
impl From<Fault> for MaskError {
    fn from(value: Fault) -> Self {
        Self::Reference(value)
    }
}

#[derive(Clone)]
pub struct ChangeMask {
    pub address: Address,
    pub evidence: Address,
    trace: PathBuf,
    events: usize,
}
pub struct ChangeMasks {
    storage: Trie,
    trace_root: PathBuf,
    serial: u64,
}
impl Default for ChangeMasks {
    fn default() -> Self {
        let session = MASK_SESSION.fetch_add(1, Ordering::Relaxed);
        let trace_root = std::env::var_os("NATIVE_TRIE_TRACE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("native-trie-traces"))
            .join("masks")
            .join(format!("{}-{session}", std::process::id()));
        fs::create_dir_all(&trace_root).expect("create mask trace directory");
        Self {
            storage: Trie::default(),
            trace_root,
            serial: 0,
        }
    }
}
impl ChangeMasks {
    pub fn root(&self) -> Address {
        self.storage.root()
    }
    pub fn trace_root(&self) -> &std::path::Path {
        &self.trace_root
    }

    fn axis(
        mut at: Address,
        name: &[u8],
        changed: bool,
        before: &Address,
        after: &Address,
    ) -> Address {
        at = at
            .provision(name)
            .provision(if changed { b"changed" } else { b"preserved" });
        if changed {
            at = at.provision(&before.identity_key());
            at.provision(b"before").point_to(before);
            at = at.provision(&after.identity_key());
            at.provision(b"after").point_to(after);
        }
        at
    }

    pub fn derive(
        &mut self,
        deltas: &[AxisDelta],
        evidence: &Address,
    ) -> Result<ChangeMask, MaskError> {
        if deltas.is_empty() {
            return Err(MaskError::Empty);
        }
        let events = self.root().provision(b"change-mask-events");
        let occurrence = self
            .root()
            .provision(b"change-masks")
            .provision(&evidence.identity_key());
        occurrence.provision(b"evidence").point_to(evidence);
        let trace = self.trace_root.join(format!("{:016x}.trace", self.serial));
        self.serial += 1;
        let mut writer = BufWriter::new(File::create(&trace).map_err(|_| MaskError::Io)?);
        writer.write_all(b"NTM1").map_err(|_| MaskError::Io)?;
        writer
            .write_all(&(deltas.len() as u64).to_be_bytes())
            .map_err(|_| MaskError::Io)?;
        for delta in deltas {
            let mut at = events.clone();
            at = Self::axis(
                at,
                b"position",
                delta.position_changed,
                &delta.input.position,
                &delta.output.position,
            );
            at = Self::axis(
                at,
                b"neighbour-position",
                delta.neighbour_position_changed,
                &delta.input.neighbour_position,
                &delta.output.neighbour_position,
            );
            at = Self::axis(
                at,
                b"color",
                delta.color_changed,
                &delta.input.color,
                &delta.output.color,
            );
            at = Self::axis(
                at,
                b"neighbour-color",
                delta.neighbour_color_changed,
                &delta.input.neighbour_color,
                &delta.output.neighbour_color,
            );
            writer
                .write_all(&at.provision(b"complete").identity_key())
                .map_err(|_| MaskError::Io)?;
        }
        writer.flush().map_err(|_| MaskError::Io)?;
        occurrence
            .provision(b"trace-file")
            .provision(trace.as_os_str().as_encoded_bytes())
            .terminate();
        Ok(ChangeMask {
            address: occurrence,
            evidence: evidence.clone(),
            trace,
            events: deltas.len(),
        })
    }

    fn bytes(mask: &ChangeMask) -> Result<Vec<u8>, MaskError> {
        let mut reader = BufReader::new(File::open(&mask.trace).map_err(|_| MaskError::Io)?);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).map_err(|_| MaskError::Io)?;
        if bytes.len() != 12 + mask.events * std::mem::size_of::<usize>() || &bytes[..4] != b"NTM1"
        {
            return Err(MaskError::Corrupt);
        }
        Ok(bytes)
    }

    pub fn matches(expected: &ChangeMask, actual: &ChangeMask) -> bool {
        matches!((Self::bytes(expected), Self::bytes(actual)), (Ok(a), Ok(b)) if a == b)
    }
}
