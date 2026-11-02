# Kaggle execution receipt

Candidate 002 was attached as a private Kaggle dataset and executed by the private, Internet-disabled notebook `bakobiibizo/trie-native-arc-2026-submission`.

## Terminal run

| Field | Value |
|---|---|
| Kernel version | 4 |
| Status | Complete |
| Wall time | 867.317 seconds |
| Tasks | 240 |
| Queries | 259 |
| Submission schema | Valid |
| Learned attempt 1 | 69 |
| Copy-input fallback | 190 |
| Independent attempt 2 | 5 |
| Task statuses | 235 ok, 4 timeout, 1 worker error |
| Submission SHA-256 | `21c15d35d2c4c226973c454c64a2a9cd27eeccfb6c0c8c7e6d2799ba939c8101` |
| Frozen predictions SHA-256 | `def22cc3b2610a79265bd755ffcec22b5dad37762fa65712bdf0a75c8e161090` |

Versions 1–3 failed before worker execution because the current Kaggle image mounts attached resources beneath `/kaggle/input/datasets` and `/kaggle/input/competitions`, rather than the legacy direct slug paths. Version 4 discovers exactly one bundle manifest and one challenge file beneath those bounded mount roots. Those failed runs were not converted into successful evidence.

No development or evaluation solution file was attached to the notebook. Hidden accuracy was not available at verification time.
