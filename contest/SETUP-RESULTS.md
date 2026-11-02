# Local setup verdict — no publication or submission

Origin operator explicitly requests setup but **do not submit anything**; public release depends on partner approval and a sufficiently good score (threshold not specified). This implementation contains no Kaggle API, authentication, upload, publish or submission operation. Everything below is LOCAL private preparation. Research candidate remains native `--source-groups`; no new solver/score is claimed.

## Implemented

- `arc_submission.py`: all task IDs/all test outputs in original order, exact two grid attempts, schema validation. One native answer duplicated, not two independent solves. Unsupported/missing/invalid/error/timeout/budget cases get explicitly labelled copy-input baselines OUTSIDE the native learner; raw native rejections/failures stay intact. No retry/acceptance/memory-limit relaxation.
- Full/partial native stderr proofs stream to gzip with raw SHA and reference closure checks. Failed worker is not successful merely because its partial graph closes. Source/challenge/worker hashes and per-query origin/status preserved; immutable freeze precedes separate solution loading/scoring. Entire output denominator used, learned and baseline exact scores separate; malformed coverage fails, not filtered.
- `offline_submission.ipynb`: unexecuted source notebook, explicit asset/data/work paths, no network/install/build/API calls. Matching prebuilt worker verified/copied to working storage; native inputs contain ALL train pairs and test INPUTS only. Kaggle UI Internet OFF remains operator configuration, not certified by source code.
- `build_bundle.py`: pinned complete native source/runner/tests/notebook plus ARM and static x86_64-musl assets; no competition data, LDGR history or credentials. Bundle is local only; no authorization to distribute. Rebuild uses Cargo offline/locked/empty external dependencies and disables rustup auto-install.

## Actual evidence read

`evidence/contest-local-setup-001/`:
- 15 Python orchestration/validation tests passed (explicit mock workers, NOT native learning evidence). Covers multiple task/test outputs/all train pairs, path safety, exact attempts, invalid grids/graphs/stdout IDs, no-match/failure/timeout/budget differentiation, frozen scoring/full denominator, challenge-gold rejection, duplicate JSON keys, no overwrites, tampered/unsupported bundle.
- 210 Rust tests passed. All 101 native source-file hashes match the previously recorded complete-count baseline: learner/kernel unchanged.
- Offline real-native notebook code-cell execution in a fresh Python process, with Python network calls blocked: 3 controlled tasks/4 query outputs, valid submission.json, 2 learned predictions/2 explicit baselines, all 3 native workers status0 and closed graphs; CRC/decompressed hashes verified. Two learned exact, two baseline incorrect. **This controlled 2/4 is NOT ARC validation/leaderboard accuracy.** Solutions loaded after freeze.
- ARM binary executed locally. x86_64 static-musl binary cross-built and ELF machine/linkage inspected. No x86 emulator available; x86 execution and actual Kaggle runtime are **unverified**. Initial missing-std (wrong active toolchain) and host-cc -m64 link failures retained; corrected with explicit Rust1.95 target and its bundled LLD, no system permission/network configuration change or external Cargo crate. Official Rust std target components were downloaded locally; notebook needs no download.

## Artifacts

- Notebook: `contest/offline_submission.ipynb`
- Bundle + receipt: `evidence/contest-local-setup-001/assets/native-arc-local.zip` and `bundle-receipt.json`
- Local controlled example ONLY: `evidence/contest-local-setup-001/notebook-validation/working/submission.json`
- Notebook run receipt: `.../notebook-validation/validation.json`; native logs/proofs/freeze/scores in `.../working/local-notebook-run/`.

Defaults worker30s/prior envelope and cohort36000s/10h/chosen nominal 2h headroom below provided12h venue limit are recorded at use. They are NOT a hard whole-process/Kaggle timing guarantee; build/load/final validation/scoring/scheduling outside cohort wait/admission budget. Native286720000-byte owned-tree cap remains unchanged, not RSS; Python/source/import/record/export separate. Current historical1/51 remains development data, not contest performance. Partner-ready threshold remains UNSET.

Local setup acceptance met. Kaggle deployment, public notebook eligibility, actual platform timing/storage/network settings and an independent score remain unverified, NOT reclassified as completed validation. No upload/publication/submission occurred. Future partner/publication/submission decisions require explicit operator approval. Pending research: native-property-action-partitions; change pinned worker only with new provenance and evaluation, never relabel baseline guesses.
