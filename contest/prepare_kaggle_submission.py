"""Stage and locally validate private Kaggle ARC code-submission assets; never upload."""
import argparse
import json
import pathlib
import platform
import shutil
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "contest"))
from arc_submission import load_challenges, load_json_unique, sha256, validate_submission, write_json
from build_bundle import build
from notebook_support import verify_bundle

COMPETITION = "arc-prize-2026-arc-agi-2"
DATASET_SLUG = "trie-native-arc-submission"
KERNEL_SLUG = "trie-native-arc-2026-submission"


def validate_sample(challenges, sample):
    if not isinstance(sample, dict) or list(sample) != list(challenges):
        raise ValueError("Sample task IDs/order differ from official challenges")
    for task, challenge in challenges.items():
        answers = sample[task]
        if not isinstance(answers, list) or len(answers) != len(challenge["test"]):
            raise ValueError(f"{task}: sample query count differs from challenges")
        for index, answer in enumerate(answers):
            if not isinstance(answer, dict) or set(answer) != {"attempt_1", "attempt_2"}:
                raise ValueError(f"{task}/{index}: sample does not define exactly two attempts")


def stage(output_dir, owner, challenges_path, sample_path):
    output_dir = pathlib.Path(output_dir).resolve()
    challenges_path = pathlib.Path(challenges_path).resolve()
    sample_path = pathlib.Path(sample_path).resolve()
    if output_dir.exists():
        raise FileExistsError("Preserve an existing staging directory; choose a fresh path")
    if not owner or "/" in owner or owner in (".", ".."):
        raise ValueError("Kaggle owner must be one plain account slug")

    challenges = load_challenges(challenges_path)
    sample = load_json_unique(sample_path)
    validate_sample(challenges, sample)

    output_dir.mkdir(parents=True)
    build_dir = output_dir / "build"
    bundle = build(build_dir)
    bundle_manifest = verify_bundle(bundle)

    dataset = output_dir / "dataset"
    dataset.mkdir()
    shutil.copyfile(build_dir / "native-arc-submission.zip", dataset / "native-arc-submission.zip")
    shutil.copyfile(build_dir / "bundle-receipt.json", dataset / "bundle-receipt.json")
    write_json(dataset / "dataset-metadata.json", {
        "title": "Trie Native ARC Submission Assets",
        "id": f"{owner}/{DATASET_SLUG}",
        "licenses": [{"name": "other"}],
    })

    kernel = output_dir / "kernel"
    kernel.mkdir()
    notebook_name = "trie-native-arc-submission.ipynb"
    shutil.copyfile(ROOT / "contest/offline_submission.ipynb", kernel / notebook_name)
    kernel_metadata = {
        "id": f"{owner}/{KERNEL_SLUG}",
        "title": "Trie Native ARC 2026 Submission",
        "code_file": notebook_name,
        "language": "python",
        "kernel_type": "notebook",
        "is_private": "true",
        "enable_gpu": "false",
        "enable_tpu": "false",
        "enable_internet": "false",
        "machine_shape": "",
        "dataset_sources": [f"{owner}/{DATASET_SLUG}"],
        "competition_sources": [COMPETITION],
        "kernel_sources": [],
        "model_sources": [],
    }
    write_json(kernel / "kernel-metadata.json", kernel_metadata)

    validation = output_dir / "official-local-validation"
    validation.mkdir()
    notebook_config = {
        "bundle": str(dataset / "native-arc-submission.zip"),
        "bundle_receipt": str(dataset / "bundle-receipt.json"),
        "challenges": str(challenges_path),
        "work": str(validation / "work"),
        "run_name": "official-test-local-run",
        "solutions": None,
        "worker_timeout_seconds": 30,
        "cohort_budget_seconds": 36000,
    }
    execution = validation / "execute-notebook.py"
    execution.write_text(
        "import json,pathlib,socket\n"
        "def denied(*args,**kwargs): raise RuntimeError('Network forbidden in local notebook validation')\n"
        "socket.socket=denied\nsocket.create_connection=denied\n"
        f"CONFIG={notebook_config!r}\n"
        f"notebook=json.loads(pathlib.Path({str(kernel / notebook_name)!r}).read_text())\n"
        "namespace={'CONFIG':CONFIG}\n"
        "for cell in notebook['cells']:\n"
        " if cell['cell_type']=='code': exec(compile(''.join(cell['source']),cell['id'],'exec'),namespace)\n"
    )
    with (validation / "notebook.stdout").open("wb") as stdout, (validation / "notebook.stderr").open("wb") as stderr:
        process = subprocess.run([sys.executable, str(execution)], cwd=validation, stdout=stdout,
                                 stderr=stderr, timeout=36600)
    write_json(validation / "notebook-status.json", {"return_code": process.returncode, "supervisor_timeout_seconds": 36600})
    if process.returncode:
        raise RuntimeError("Local execution of staged notebook failed; inspect preserved stdout/stderr")
    manifest = load_json_unique(validation / "work/official-test-local-run/manifest.json")
    submission = load_json_unique(validation / "work/submission.json")
    validate_submission(challenges, submission)
    if list(submission) != list(sample):
        raise AssertionError("Generated submission task order differs from official sample")
    for task in challenges:
        if len(submission[task]) != len(sample[task]):
            raise AssertionError(f"{task}: generated query count differs from official sample")

    notebook = load_json_unique(kernel / notebook_name)
    for cell in notebook.get("cells", []):
        if cell.get("cell_type") == "code":
            if cell.get("outputs") != [] or cell.get("execution_count") is not None:
                raise ValueError("Staged notebook must be unexecuted")
            compile("".join(cell["source"]), cell.get("id", "notebook-cell"), "exec")

    instructions = (
        "PRIVATE PRE-SUBMISSION STAGING — NO UPLOAD HAS RUN.\n\n"
        "After a separate explicit operator instruction, create the private dataset first, then push "
        "the private notebook. Review the resulting notebook version before any competition submission.\n\n"
        f"Dataset staging: {dataset}\nKernel staging: {kernel}\n"
    )
    (output_dir / "REVIEW-BEFORE-UPLOAD.txt").write_text(instructions)
    receipt = {
        "scope": "Private local Kaggle code-submission staging and official-input format/runtime validation",
        "competition": COMPETITION,
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "platform": platform.platform(),
        "challenges_sha256": sha256(challenges_path),
        "sample_sha256": sha256(sample_path),
        "bundle_manifest_sha256": sha256(bundle / "bundle-manifest.json"),
        "bundle_archive_sha256": sha256(dataset / "native-arc-submission.zip"),
        "notebook_sha256": sha256(kernel / notebook_name),
        "kernel_metadata_sha256": sha256(kernel / "kernel-metadata.json"),
        "dataset_metadata_sha256": sha256(dataset / "dataset-metadata.json"),
        "submission_sha256": sha256(validation / "work/submission.json"),
        "tasks": manifest["tasks"],
        "queries": manifest["queries"],
        "learned_queries": manifest["learned_queries"],
        "baseline_queries": manifest["baseline_queries"],
        "task_status_counts": manifest["task_status_counts"],
        "local_wall_seconds": manifest["wall_seconds"],
        "x86_worker": bundle_manifest["workers"]["linux-x86_64-musl"],
        "format_valid": True,
        "sample_structure_matches": True,
        "hidden_output_accuracy_known": False,
        "kaggle_runtime_verified": False,
        "dataset_uploaded": False,
        "kernel_uploaded": False,
        "submitted": False,
    }
    write_json(output_dir / "staging-receipt.json", receipt)
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--owner", required=True)
    parser.add_argument("--challenges", required=True)
    parser.add_argument("--sample", required=True)
    args = parser.parse_args()
    receipt = stage(args.output_dir, args.owner, args.challenges, args.sample)
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
