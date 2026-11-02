# ARC Prize 2026 private code-submission preparation

Origin: operator approval to prepare and locally validate a private code submission for
`arc-prize-2026-arc-agi-2`. Uploading, publishing, creating a Kaggle dataset/notebook version,
or submitting that version still requires a separate explicit operator instruction.

## Submission shape

The staged submission consists of two private Kaggle resources:

1. A private dataset containing one hash-pinned offline asset archive and its receipt. The
   archive contains the static x86_64-musl `sparse_probe` worker, complete source provenance,
   the Python adapter, and the notebook support code.
2. A private Internet-disabled notebook attached to that dataset and the competition. It safely
   extracts and verifies the archive, selects the matching worker, runs every official test task
   using every training pair and test input, and writes `/kaggle/working/submission.json`.

The notebook does not compile, install packages, access the network, inspect expected test
outputs, or invoke Kaggle APIs. Every output has exactly `attempt_1` and `attempt_2`. Attempt 2 may
use the first output-distinct separately addressed construction retained by the same induction
pass, but only after that construction independently replayed every demonstration and passes the
query applicability and application guards. Absence, inapplicability, malformed
provenance/output, or output identity honestly duplicates attempt 1. A native abstention, failure, timeout, malformed primary result, or
budget omission receives a copy-input format fallback outside the learner.

## Current solver boundary

The worker uniformly invokes `SparseRewrite::agree_generalized_candidates`; the historical
`agree_generalized_actions` API remains available and returns its unchanged first candidate. The
candidate set retains replay-valid BROAD alternatives from one complete-demonstration pass and
never reruns induction per query. There is no task-ID or query-gold dispatch. Candidate parameters are
induced from each task's complete training evidence
and must replay every training output exactly before query execution. Construction schemas,
connectivity, grouping, candidate order, and renderers remain host-coded Rust vocabulary.

The previous 45/51 score is development-cohort evidence, not an estimate for this competition's
unseen outputs. Official test challenges have no solutions, so local preparation can establish
only completeness, runtime, provenance, and submission-schema validity—not leaderboard accuracy.

## Local acceptance criteria

`contest/prepare_kaggle_submission.py` must, without any upload:

- build the current host worker and static x86_64-musl worker offline and locked;
- hash-pin every bundled file and verify worker ELF architecture/linkage;
- stage private dataset and notebook metadata with Internet disabled and the intended competition;
- run the real bundled host worker over the downloaded official test challenges;
- match all sample-submission task IDs, query counts, ordering, and attempt keys;
- preserve per-query native-versus-fallback provenance and worker proof streams;
- record source, challenge, sample, bundle, notebook, and resulting submission hashes.

Python orchestration tests, the complete native Rust suite, and all-target Clippy with warnings
denied must pass before the staged candidate is called ready for operator review.

## Explicit non-actions

Preparation does **not** run `kaggle datasets create`, `kaggle datasets version`,
`kaggle kernels push`, or `kaggle competitions submit`. The staging receipt must record these
states as false. Private upload and code submission are separate operator-authorized transitions.
