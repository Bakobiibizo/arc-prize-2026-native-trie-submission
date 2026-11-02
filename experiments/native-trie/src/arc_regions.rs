//! Learned output associations keyed by actual native region proofs and supplied color traits.
//! No host flood fill, role labels, image operation, or decoding of witness outcome tags.
use crate::{
    Address,
    arc_colors::{Choice, ColorError, ColorModel, ColorRun},
    arc_grids::{Grid, GridError, Grids},
    records,
    region_witness::{RegionError, RegionWitness},
};
use std::time::Instant;
impl From<RegionError> for ColorError {
    fn from(e: RegionError) -> Self {
        Self::Region(e)
    }
}
#[derive(Default)]
struct Stats {
    copies: u64,
    setup: u128,
    execute: u128,
    hits: usize,
    proofs: usize,
}
pub struct RegionModel {
    pub base: ColorModel,
    serial: usize,
    fact_views: usize,
    activation_views: usize,
    pub max_views: usize,
    pub max_fact_views: usize,
}
impl RegionModel {
    pub fn new(grids: &Grids, scope: &[u8]) -> Result<Self, ColorError> {
        Ok(Self {
            base: ColorModel::new(grids, scope)?,
            serial: 0,
            fact_views: 0,
            activation_views: 0,
            max_views: 8192,
            max_fact_views: 32768,
        })
    }
    fn derive(
        &mut self,
        input: &Grid,
        cell: &Address,
        stats: &mut Stats,
    ) -> Result<(Address, Address), ColorError> {
        let cache = self
            .base
            .root
            .provision(b"witness-cache")
            .provision(&input.anchor.identity_key());
        if let Some(hit) = RegionWitness::cached(&cache, cell)? {
            stats.hits += 1;
            return Ok(hit);
        }
        let count = input.width() * input.height() + 1;
        // Conservative physical storage proxies measured in region-retirement-development-001.
        // Failed activations stay charged; success releases code but retains native fact nodes.
        let active = self.activation_views + count;
        let facts = self.fact_views + count;
        if active > self.max_views
            || facts > self.max_fact_views
            || active as u128 * 35_000 + facts as u128 * 7168 > self.max_views as u128 * 35_000
        {
            return Err(ColorError::WorkspaceLimit);
        }
        self.activation_views += count;
        let start = Instant::now();
        let mut w = RegionWitness::prepare_bound(
            input,
            cell,
            &self.base.root,
            &self.serial.to_be_bytes(),
            &cache,
        )?;
        self.serial += 1;
        stats.setup += start.elapsed().as_nanos();
        let start = Instant::now();
        let copies = w.execute(w.budget)?;
        stats.execute += start.elapsed().as_nanos();
        stats.copies += copies;
        stats.proofs += 1;
        let p = w.record()?;
        self.activation_views -= count;
        self.fact_views += p.recorded_views.max(1);
        Ok((p.proof, p.value))
    }
    pub fn teach(&mut self, id: &[u8], input: &Grid, output: &Grid) -> Result<(), ColorError> {
        self.base.teach(id, input, output)?;
        let episode = self
            .base
            .root
            .child(b"examples")
            .and_then(|a| a.child(id))
            .ok_or(GridError::Malformed)?;
        let mut stats = Stats::default();
        for y in 0..input.height() {
            for x in 0..input.width() {
                let before = input.cell(y, x)?;
                let after = output.cell(y, x)?;
                let (proof, kind) = self.derive(input, &before, &mut stats)?;
                let value = before
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                let out = after
                    .child(b"value")
                    .ok_or(GridError::Malformed)?
                    .follow()?;
                let anchor = self
                    .base
                    .root
                    .provision(b"region-context")
                    .provision(&kind.identity_key())
                    .provision(&value.identity_key());
                anchor
                    .provision(b"construction")
                    .point_to(&self.base.construction);
                let evidence = episode
                    .provision(&y.to_be_bytes())
                    .provision(&x.to_be_bytes())
                    .provision(b"region");
                evidence.provision(b"input-cell").point_to(&before);
                evidence.provision(b"output-cell").point_to(&after);
                evidence.provision(b"source-proof").point_to(&proof);
                evidence.provision(b"kind").point_to(&kind);
                records::record(&anchor, b"output", &out, &evidence)?;
            }
        }
        Ok(())
    }
    pub fn begin(&mut self, id: &[u8], input: &Grid) -> Result<ColorRun, ColorError> {
        let mut run = self.base.begin(id, input)?;
        let mut stats = Stats::default();
        for cell in &run.cells {
            let (proof, kind) = self.derive(input, &cell.input, &mut stats)?;
            cell.node.provision(b"region-proof").point_to(&proof);
            cell.node.provision(b"region-kind").point_to(&kind);
        }
        run.relation_copies = stats.copies;
        run.relation_setup_ns = stats.setup;
        run.relation_execute_ns = stats.execute;
        run.relation_cache_hits = stats.hits;
        run.relation_proofs = stats.proofs;
        Ok(run)
    }
    pub fn execute(&mut self, run: &mut ColorRun, limit: usize) -> Result<(), ColorError> {
        let root = self.base.root.clone();
        self.base.execute_with(run, limit, |cell| {
            let kind = cell
                .node
                .child(b"region-kind")
                .ok_or(GridError::Malformed)?
                .follow()?;
            let value = cell
                .input
                .child(b"value")
                .ok_or(GridError::Malformed)?
                .follow()?;
            let exact = root
                .child(b"region-context")
                .and_then(|a| a.child(&kind.identity_key()))
                .and_then(|a| a.child(&value.identity_key()));
            if let Some(at) = exact {
                cell.lookup.point_to(&at);
                return Ok(Choice::from(records::resolve(&at, b"output")?));
            }
            // Only an absent refinement can use independently supported trait evidence; never bypass a contradiction.
            match root
                .child(b"traits")
                .and_then(|a| a.child(b"color"))
                .and_then(|a| a.child(&value.identity_key()))
            {
                Some(at) => {
                    cell.lookup.point_to(&at);
                    Ok(records::resolve(&at, b"output")?.into())
                }
                None => Ok(Choice::Unknown),
            }
        })
    }
    pub fn record(&self, grids: &mut Grids, run: ColorRun) -> Result<Grid, ColorError> {
        self.base.record(grids, run)
    }
}
