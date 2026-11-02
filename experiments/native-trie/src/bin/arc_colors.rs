#[path = "../../support/heap.rs"]
mod heap;
#[path = "../../support/arc_pattern_trace.rs"]
mod pattern_trace;
#[path = "../../support/arc_region_trace.rs"]
mod region_trace;
use native_trie_seed::{
    arc_colors::{ColorError, ColorModel, ColorRun},
    arc_grids::{Grid, Grids, Side},
    arc_markers::MarkerModel,
    arc_patterns::PatternModel,
    arc_regions::RegionModel,
};
use std::{fmt::Write, fs, time::Instant};
fn grid(text: &str) -> Vec<Vec<Option<u8>>> {
    text.split('/')
        .map(|r| {
            r.split(',')
                .map(|v| Some(v.parse().expect("supplied cell integer")))
                .collect()
        })
        .collect()
}
fn json(grid: &[Vec<Option<u8>>]) -> String {
    let mut out = String::from("[");
    for (y, row) in grid.iter().enumerate() {
        if y > 0 {
            out.push(',')
        }
        out.push('[');
        for (x, v) in row.iter().enumerate() {
            if x > 0 {
                out.push(',')
            }
            match v {
                Some(v) => write!(out, "{v}").unwrap(),
                None => out.push_str("null"),
            }
        }
        out.push(']')
    }
    out.push(']');
    out
}
// The caller explicitly selects a family. No task inspection chooses/tries an operator catalogue.
enum Model {
    Colors(ColorModel),
    Patterns(PatternModel),
    Regions(RegionModel),
    Markers(Box<MarkerModel>),
}
impl Model {
    fn teach(&mut self, id: &[u8], a: &Grid, b: &Grid) -> Result<(), ColorError> {
        match self {
            Self::Colors(m) => m.teach(id, a, b),
            Self::Patterns(m) => m.teach(id, a, b),
            Self::Regions(m) => m.teach(id, a, b),
            Self::Markers(m) => m.teach(id, a, b),
        }
    }
    fn begin(&mut self, id: &[u8], a: &Grid) -> Result<ColorRun, ColorError> {
        match self {
            Self::Colors(m) => m.begin(id, a),
            Self::Patterns(m) => m.begin(id, a),
            Self::Regions(m) => m.begin(id, a),
            Self::Markers(m) => m.begin(id, a),
        }
    }
    fn execute(&mut self, run: &mut ColorRun, limit: usize) -> Result<(), ColorError> {
        match self {
            Self::Colors(m) => m.execute(run, limit),
            Self::Patterns(m) => m.execute(run, limit),
            Self::Regions(m) => m.execute(run, limit),
            Self::Markers(m) => m.execute(run, limit),
        }
    }
    fn record(&self, s: &mut Grids, run: ColorRun) -> Result<Grid, ColorError> {
        match self {
            Self::Colors(m) => m.record(s, run),
            Self::Patterns(m) => m.record(s, run),
            Self::Regions(m) => m.record(s, run),
            Self::Markers(m) => m.record(s, run),
        }
    }
    fn marker_stats(&self, phase: &str, id: &str) {
        if let Self::Markers(m) = self {
            let s = &m.last;
            eprintln!(
                "{{\"kind\":\"marker-stats\",\"phase\":\"{phase}\",\"id\":\"{id}\",\"region_copies\":{},\"role_copies\":{},\"expression_copies\":{},\"region_setup_ns\":{},\"region_execute_ns\":{},\"role_setup_ns\":{},\"role_execute_ns\":{},\"expression_setup_ns\":{},\"expression_execute_ns\":{},\"guard_ns\":{},\"peak_estimated_bytes\":{}}}",
                s.region_copies,
                s.role_copies,
                s.expression_copies,
                s.region_setup_ns,
                s.region_execute_ns,
                s.role_setup_ns,
                s.role_execute_ns,
                s.expression_setup_ns,
                s.expression_execute_ns,
                s.guard_ns,
                s.peak_charge
            );
        }
    }
    fn marker_graph(&self) {
        if let Self::Markers(m) = self {
            let start = Instant::now();
            let nodes = native_trie_seed::graph_export::write(
                &[
                    m.base.root.clone(),
                    m.base.machine.states.root(),
                    m.base.machine.output.root(),
                    m.comparison.machine.states.root(),
                    m.comparison.machine.output.root(),
                ],
                &mut std::io::stderr().lock(),
            )
            .unwrap();
            eprintln!(
                "{{\"kind\":\"graph-export\",\"nodes\":{nodes},\"ns\":{}}}",
                start.elapsed().as_nanos()
            );
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!(
        args.len() == 2
            || (args.len() == 3
                && [
                    "--patterns",
                    "--regions",
                    "--markers",
                    "--source-constraints"
                ]
                .contains(&args[2].as_str())),
        "input-only bundle path and optional --patterns/--regions/--markers/--source-constraints required"
    );
    let text = fs::read_to_string(&args[1]).unwrap();
    let start = Instant::now();
    let mut grids = Grids::new();
    let mut model = match args.get(2).map(String::as_str) {
        Some("--patterns") => Model::Patterns(PatternModel::new(&grids, b"puzzle").unwrap()),
        Some("--regions") => Model::Regions(RegionModel::new(&grids, b"puzzle").unwrap()),
        Some("--source-constraints") => Model::Markers(Box::new(
            MarkerModel::new_constraints(&grids, b"puzzle").unwrap(),
        )),
        Some("--markers") => Model::Markers(Box::new(MarkerModel::new(&grids, b"puzzle").unwrap())),
        _ => Model::Colors(ColorModel::new(&grids, b"puzzle").unwrap()),
    };
    let setup_ns = start.elapsed().as_nanos();
    let mut teaching_ns = 0;
    let mut queried = false;
    for line in text.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let id = fields[1];
        assert!(id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'));
        if fields[0] == "TRAIN" {
            assert!(!queried, "model must freeze before queries");
            assert_eq!(fields.len(), 4);
            let start = Instant::now();
            let source = grids.source(id.as_bytes(), line.as_bytes()).unwrap();
            let before = grids
                .admit(
                    id.as_bytes(),
                    Side::TrainingInput,
                    &grid(fields[2]),
                    &source,
                )
                .unwrap();
            let after = grids
                .admit(
                    id.as_bytes(),
                    Side::TrainingOutput,
                    &grid(fields[3]),
                    &source,
                )
                .unwrap();
            let taught = model.teach(id.as_bytes(), &before, &after);
            model.marker_stats("teach", id);
            if taught.is_err() {
                model.marker_graph();
            }
            taught.unwrap();
            teaching_ns += start.elapsed().as_nanos();
        } else {
            assert_eq!(fields[0], "QUERY");
            assert_eq!(fields.len(), 3);
            queried = true;
            let start = Instant::now();
            let source = grids.source(id.as_bytes(), line.as_bytes()).unwrap();
            let input = grids
                .admit(id.as_bytes(), Side::QueryInput, &grid(fields[2]), &source)
                .unwrap();
            let input_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            heap::start();
            let begun = model.begin(id.as_bytes(), &input);
            if begun.is_err() {
                model.marker_stats("query-failed-begin", id);
                model.marker_graph();
            }
            let mut run = begun.unwrap();
            let workspace_ns = start.elapsed().as_nanos();
            let workspace = heap::finish();
            heap::start();
            let start = Instant::now();
            let executed = model.execute(&mut run, input.width() * input.height());
            if executed.is_err() {
                model.marker_stats("query-failed-execute", id);
                model.marker_graph();
            }
            executed.unwrap();
            let execution_ns = start.elapsed().as_nanos();
            let execution = heap::finish();
            assert_eq!(execution.requests, 0);
            assert_eq!(execution.bytes, 0);
            let relation_copies = run.relation_copies;
            let relation_setup_ns = run.relation_setup_ns;
            let relation_execute_ns = run.relation_execute_ns;
            let relation_cache_hits = run.relation_cache_hits;
            let relation_proofs = run.relation_proofs;
            let cycles = run.cycles;
            let unknown = run.unknown;
            let ambiguous = run.ambiguous;
            heap::start();
            let start = Instant::now();
            let result = model.record(&mut grids, run).unwrap();
            let recording_ns = start.elapsed().as_nanos();
            let recording = heap::finish();
            let start = Instant::now();
            let output = result.inspect().unwrap();
            let inspection_ns = start.elapsed().as_nanos();
            let actual = json(&output);
            println!(
                "{{\"id\":\"{id}\",\"grid\":{actual},\"copy_cycles\":{cycles},\"unknown\":{unknown},\"ambiguous\":{ambiguous},\"setup_ns\":{setup_ns},\"teaching_ns\":{teaching_ns},\"input_ns\":{input_ns},\"workspace_ns\":{workspace_ns},\"execution_ns\":{execution_ns},\"execution_allocations\":{},\"recording_ns\":{recording_ns},\"recording_allocations\":{},\"recording_bytes\":{},\"inspection_ns\":{inspection_ns},\"relation_copies\":{relation_copies},\"relation_setup_ns\":{relation_setup_ns},\"relation_execute_ns\":{relation_execute_ns},\"relation_cache_hits\":{relation_cache_hits},\"relation_proofs\":{relation_proofs},\"workspace_requests\":{},\"workspace_bytes\":{}}}",
                execution.requests,
                recording.requests,
                recording.bytes,
                workspace.requests,
                workspace.bytes
            );
            model.marker_stats("query", id);
            model.marker_graph();
            if args.get(2).is_some_and(|a| a == "--patterns") {
                pattern_trace::trace(id, &result);
            }
            if args.get(2).is_some_and(|a| a == "--regions") {
                region_trace::trace(id, &result);
            }
        }
    }
}
