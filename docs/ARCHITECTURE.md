# Architecture and limitations

## Candidate admission

Each ARC task is admitted into addressed grid and episode structures. The learner derives one or more structural candidate programs from demonstrations. A candidate is publishable only if executing that same construction exactly reproduces every demonstration output.

Candidate 002 includes canonical mechanisms for masks, axis projections, segments, relative stencils, dihedral roles, periodic coordinates, envelopes, palette roles, connected objects, repeated blocks, separator compaction, and whole-grid transforms. Attempt 2 is selected only from an independently represented replay-valid construction that produces a distinct query output; otherwise it duplicates attempt 1.

## Trie substrate

The implementation uses trie addresses for values, roles, records, references, construction identities, and replay evidence. Large ordered occurrences and execution traces remain disk-backed.

## Honest boundaries

- Construction schemas and semantic admission are still host-authored Rust bootstrap code; they are not fully lowered into trie-held continuations.
- Learned instances are puzzle-local in this submitted worker.
- Copy-input fallback is outside the learner and is explicitly reported as such.
- Exact demonstration replay prevents unsupported candidates from being published, but does not establish query correctness.
- The Kaggle run encountered four task timeouts and one worker error. The submission remained structurally complete because those tasks used the declared fallback.

The run statistics in this repository are execution evidence, not hidden leaderboard accuracy claims.
