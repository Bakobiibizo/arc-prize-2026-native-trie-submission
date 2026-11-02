# Reproducing candidate 002

## Requirements

- Rust toolchain from `rust-toolchain.toml`
- `x86_64-unknown-linux-musl` Rust target for the Kaggle worker
- Python 3.11 or newer
- Kaggle ARC challenge JSON for execution; competition data is not redistributed here

## Build

```bash
rustup target add x86_64-unknown-linux-musl
cargo build \
  --locked \
  --release \
  --target x86_64-unknown-linux-musl \
  --manifest-path experiments/native-trie/Cargo.toml \
  --bin sparse_probe
```

The staged worker was stripped and copied to `workers/linux-x86_64-musl/sparse_probe`. Its exact submitted hash is:

```text
2717dec75a1fc1d2d457353078bd10fea6fa0d0132ae3a43d63f84c75abff8f7
```

A fresh build should be behaviorally tested; byte identity can depend on the Rust toolchain and linker environment.

## Run outside Kaggle

```bash
python3 contest/arc_submission.py \
  --challenges /path/to/arc-agi_test_challenges.json \
  --worker workers/linux-x86_64-musl/sparse_probe \
  --output submission.json \
  --artifacts run-artifacts \
  --worker-timeout-seconds 30 \
  --cohort-budget-seconds 36000
```

The runner validates all grids and the exact `attempt_1`/`attempt_2` structure before atomically writing `submission.json`.

## Notebook

`notebook/trie-native-arc-submission.ipynb` is the notebook that completed as Kaggle kernel version 4. It:

1. discovers one hash-pinned bundle under the attached dataset mount;
2. discovers one official test challenge file under the attached competition mount;
3. verifies every bundle-manifest entry and the native worker hash;
4. copies the static worker into `/kaggle/working` and executes all tasks;
5. writes `/kaggle/working/submission.json`.

Internet access is disabled in kernel metadata. The notebook contains no credentials or solution data.
