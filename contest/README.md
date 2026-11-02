# ARC Prize 2026 private code-submission tooling

Origin: operator approval to prepare and locally validate a private code submission for
`arc-prize-2026-arc-agi-2`. See `SUBMISSION-PREPARATION.md` for the current specification.
Uploading a dataset/notebook or submitting a notebook version still requires a separate explicit
operator instruction.

Historical local-only setup evidence and its older solver result remain preserved in
`SETUP-RESULTS.md` and `DEVELOPMENT-RUN-RESULTS.md`; those documents describe their own revisions,
not the current generalized inducer.

## Current candidate

The notebook uses the native `sparse_probe --cyclic-lattice-actions` (the generalized induction entrypoint) path uniformly for every task and all
training pairs. A construction is published only after exact replay of every training output. No
task identifier or query gold enters induction or routing. Native abstention, failure, timeout,
invalid evidence, or budget omission receives an explicitly labelled copy-input format fallback
outside the learner.

Each answer contains exactly `attempt_1` and `attempt_2`. Attempt 1 remains the historical
minimum-operation construction. Attempt 2 uses the first output-distinct separately addressed, all-demonstration-replayed
alternative from the retained generalized candidate set after every candidate passes the existing
query applicability and application guards; otherwise it honestly duplicates attempt 1 with a
recorded fallback reason. Prediction/status/provenance streams freeze before any optional separate
solution file is loaded.

The independently scored 45/51 result is development-cohort evidence, not competition accuracy.
Official test challenges contain no outputs; local preparation can validate only completeness,
runtime, provenance, and schema.

## Build and local validation

Download official inputs into the ignored private directory:

```sh
mkdir -p contest/private-data/arc-prize-2026-arc-agi-2
kaggle competitions download arc-prize-2026-arc-agi-2 \
  -f arc-agi_test_challenges.json \
  -p contest/private-data/arc-prize-2026-arc-agi-2 --unzip
kaggle competitions download arc-prize-2026-arc-agi-2 \
  -f sample_submission.json \
  -p contest/private-data/arc-prize-2026-arc-agi-2 --unzip
```

After committing the intended source revision, stage a fresh candidate:

```sh
python contest/prepare_kaggle_submission.py \
  --output-dir target/kaggle-submission-CANDIDATE \
  --owner KAGGLE_ACCOUNT_SLUG \
  --challenges contest/private-data/arc-prize-2026-arc-agi-2/arc-agi_test_challenges.json \
  --sample contest/private-data/arc-prize-2026-arc-agi-2/sample_submission.json
```

This builds locked/offline host and static x86_64-musl workers, stages private dataset and notebook
metadata, executes the actual staged notebook locally with Python networking blocked, and runs all
240 official test tasks/259 outputs. It never invokes a Kaggle upload or submission command.
Existing output paths are refused to preserve prior evidence.

The staged resources are:

- `dataset/`: private native bundle archive, hash receipt, and dataset metadata;
- `kernel/`: unexecuted Internet-disabled notebook and private kernel metadata;
- `official-local-validation/`: notebook stdout/stderr, complete provenance, graph proofs, and
  format-valid `submission.json`;
- `staging-receipt.json`: source/input/asset/output hashes and explicit upload/submission state.

## Validation commands

```sh
python -m unittest discover -s contest/tests -v
cargo test --offline --manifest-path experiments/native-trie/Cargo.toml \
  --target-dir target/native-trie --no-fail-fast
cargo clippy --offline --manifest-path experiments/native-trie/Cargo.toml \
  --target-dir target/native-trie --all-targets -- -D warnings
```

The notebook requires Internet OFF, uses no package installation or compilation, verifies the
asset archive and every bundled file, selects only a matching Linux worker, and writes
`/kaggle/working/submission.json`. Actual Kaggle x86 runtime remains unverified until an approved
private notebook version executes there.
