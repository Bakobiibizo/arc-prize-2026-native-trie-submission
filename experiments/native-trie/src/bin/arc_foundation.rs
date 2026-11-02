#[path = "../../support/heap.rs"]
mod heap;
use native_trie_seed::{
    arc_grids::{Grids, Side},
    arc_rows::RowProgram,
};
use std::time::Instant;
fn main() {
    println!(
        "case,repeat,stage,width,setup_ns,input_ns,binding_ns,execution_ns,execution_allocations,execution_bytes,copy_cycles,recording_ns,recording_allocations,recording_bytes,inspection_ns,correct_cells"
    );
    for line in include_str!("../../curriculum/arc-row-inputs.tsv").lines() {
        for repeat in 0..3 {
            let (id, raw) = line.split_once('\t').unwrap();
            // Fixture transport only. These literal cell values/boundaries are not solver results.
            let supplied: Vec<Vec<Option<u8>>> = raw
                .split('/')
                .map(|r| r.split(' ').map(|c| Some(c.parse().unwrap())).collect())
                .collect();
            let start = Instant::now();
            let mut grids = Grids::new();
            let curriculum = grids
                .source(
                    b"curriculum",
                    include_bytes!("../../../../references/arc-transfer/arc/native/curriculum.ts"),
                )
                .unwrap();
            let mut programs = Vec::new();
            for lesson in include_str!("../../curriculum/arc-row-lessons.tsv").lines() {
                let (name, slots) = lesson.split_once('\t').unwrap();
                let mapping: Vec<usize> = slots.split(' ').map(|s| s.parse().unwrap()).collect();
                let program =
                    RowProgram::teach(&grids, name.as_bytes(), &mapping, &curriculum).unwrap();
                programs.push((name, program, mapping));
            }
            let setup_ns = start.elapsed().as_nanos();
            let start = Instant::now();
            let source = grids.source(id.as_bytes(), line.as_bytes()).unwrap();
            let mut input = grids
                .admit(id.as_bytes(), Side::QueryInput, &supplied, &source)
                .unwrap();
            let input_ns = start.elapsed().as_nanos();
            let mut expected = supplied;
            for (name, program, mapping) in &mut programs {
                // Independent observation oracle is outside model execution and its timers.
                expected = mapping.iter().map(|i| expected[*i].clone()).collect();
                let start = Instant::now();
                program.bind(&input).unwrap();
                let binding_ns = start.elapsed().as_nanos();
                heap::start();
                let start = Instant::now();
                let cycles = program.execute(mapping.len()).unwrap();
                let execution_ns = start.elapsed().as_nanos();
                let execution = heap::finish();
                assert_eq!(cycles, mapping.len() as u64);
                assert_eq!(execution.requests, 0);
                assert_eq!(execution.bytes, 0);
                heap::start();
                let start = Instant::now();
                let result = program.record(&mut grids).unwrap();
                let recording_ns = start.elapsed().as_nanos();
                let recording = heap::finish();
                let start = Instant::now();
                let rendered = result.grid.inspect().unwrap();
                assert_eq!(rendered, expected);
                let correct: usize = rendered.iter().map(Vec::len).sum();
                let inspection_ns = start.elapsed().as_nanos();
                println!(
                    "{id},{repeat},{name},{},{setup_ns},{input_ns},{binding_ns},{execution_ns},{},{},{cycles},{recording_ns},{},{},{inspection_ns},{correct}",
                    input.width(),
                    execution.requests,
                    execution.bytes,
                    recording.requests,
                    recording.bytes
                );
                eprintln!("case={id} repeat={repeat} stage={name} actual_grid={rendered:?}");
                input = result.grid;
            }
        }
    }
}
