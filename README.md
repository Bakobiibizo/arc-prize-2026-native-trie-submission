# Native-trie ARC Prize 2026 submission — candidate 002

This repository contains the code and runtime artifacts used by candidate 002 for the Kaggle competition [`arc-prize-2026-arc-agi-2`](https://www.kaggle.com/competitions/arc-prize-2026-arc-agi-2).

The solver represents grids, demonstrations, structural roles, candidate constructions, and replay evidence through a trie-oriented Rust substrate. It publishes a learned prediction only when the candidate exactly replays every demonstration. When no learned construction applies, the submission explicitly copies the query input as an outside-learner fallback.

## Exact submitted candidate

- Source revision: `aa48b4ed489504a7f163491713b0906f6aa7aa0d`
- Linux x86_64 worker: `workers/linux-x86_64-musl/sparse_probe`
- Worker SHA-256: `2717dec75a1fc1d2d457353078bd10fea6fa0d0132ae3a43d63f84c75abff8f7`
- Bundle archive SHA-256: `45e38e3bb0e08b27fda0f96812a7050fc5fa8f1252ad47c87ccf87720e640834`
- Verified Kaggle kernel: version 4 of `bakobiibizo/trie-native-arc-2026-submission`
- Kaggle-produced `submission.json` SHA-256: `21c15d35d2c4c226973c454c64a2a9cd27eeccfb6c0c8c7e6d2799ba939c8101`

The public repository does not contain competition challenge or solution data, credentials, private development data, or local execution traces.

## Kaggle verification

The Internet-disabled private verification run completed on Kaggle in 867.3 seconds and produced a structurally valid file containing all 240 task IDs and 259 query outputs:

- 69 learned attempt-1 outputs
- 190 explicit copy-input fallbacks
- 5 independent attempt-2 outputs
- 235 normal task completions
- 4 timeouts and 1 worker error, all represented through the declared fallback

These are execution and format results, not claims of hidden-set accuracy. See [`docs/KAGGLE-RUN.md`](docs/KAGGLE-RUN.md).

## Layout

- `experiments/native-trie/`: Rust computational substrate and solver source
- `contest/arc_submission.py`: bounded task runner and submission writer
- `contest/notebook_support.py`: bundle and worker verification
- `workers/linux-x86_64-musl/sparse_probe`: exact static Kaggle worker
- `notebook/trie-native-arc-submission.ipynb`: exact successful notebook version
- `bundle-manifest.json`: hashes of the staged candidate files
- `docs/REPRODUCING.md`: build and execution instructions
- `docs/ARCHITECTURE.md`: mechanism and limitation summary

## Quick verification

```bash
sha256sum workers/linux-x86_64-musl/sparse_probe
python3 -m unittest contest.tests.test_arc_submission
cargo test --locked --manifest-path experiments/native-trie/Cargo.toml
```

The notebook expects Kaggle-mounted competition data and a dataset containing the bundle. It does not use network access.

## Licensing

No license grant is included in this initial publication. The source is publicly inspectable, but reuse rights remain reserved until the repository owner selects a license.
