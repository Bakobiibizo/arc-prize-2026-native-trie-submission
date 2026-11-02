"""Execute notebook code cells locally with real native worker and Python networking blocked.
Controlled harness test, not Kaggle/leaderboard/independent ARC validation.
"""
import argparse
import gzip
import hashlib
import json
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from arc_submission import sha256, validate_submission, write_json
from notebook_support import verify_bundle, elf_info


def validate(bundle, output):
    bundle, output = pathlib.Path(bundle).resolve(), pathlib.Path(output).resolve()
    if output.exists():
        raise FileExistsError('Validation output exists; preserve evidence')
    info = verify_bundle(bundle)
    output.mkdir(parents=True)
    challenges = {
        'swap': {'train': [
            {'input': [[2, 0, 5, 8], [5, 2, 0, 8]], 'output': [[5, 0, 2, 8], [2, 5, 0, 8]]},
            {'input': [[0, 2, 8], [5, 8, 2], [8, 5, 0]], 'output': [[0, 5, 8], [2, 8, 5], [8, 2, 0]]}],
            'test': [{'input': [[5, 2, 5, 0, 8], [2, 5, 2, 8, 0]]}, {'input': [[2, 5, 8, 0]]}]},
        'conflict': {'train': [{'input': [[2]], 'output': [[5]]}, {'input': [[2]], 'output': [[4]]}],
                     'test': [{'input': [[2]]}]},
        'shape-change': {'train': [{'input': [[1]], 'output': [[1, 1]]}, {'input': [[2]], 'output': [[2, 2]]}],
                         'test': [{'input': [[3]]}]},
    }
    solutions = {'swap': [[[2, 5, 2, 0, 8], [5, 2, 5, 8, 0]], [[5, 2, 8, 0]]],
                 'conflict': [[[5]]], 'shape-change': [[[3, 3]]]}
    write_json(output / 'challenges.json', challenges)
    write_json(output / 'solutions.json', solutions)
    config = dict(bundle=str(bundle), challenges=str(output / 'challenges.json'), work=str(output / 'working'),
                  run_name='local-notebook-run', solutions=str(output / 'solutions.json'),
                  worker_timeout_seconds=30, cohort_budget_seconds=60)
    script = output / 'execute-notebook.py'
    script.write_text('''import json,pathlib,socket
# Test-only network denial. This does not certify Kaggle UI/network settings.
def denied(*args,**kwargs):raise RuntimeError('Python network forbidden in local notebook test')
socket.socket=denied
socket.create_connection=denied
try:socket.create_connection(('example.invalid',443))
except RuntimeError:pass
else:raise AssertionError('Network denial inactive')
CONFIG=''' + repr(config) + '''
notebook=json.loads((pathlib.Path(CONFIG['bundle'])/'contest/offline_submission.ipynb').read_text())
namespace={'CONFIG':CONFIG}
for cell in notebook['cells']:
 if cell['cell_type']=='code':exec(compile(''.join(cell['source']),cell['id'],'exec'),namespace)
''')
    with (output / 'notebook.stdout').open('wb') as so, (output / 'notebook.stderr').open('wb') as se:
        process = subprocess.run([sys.executable, str(script)], cwd=output, stdout=so, stderr=se, timeout=90)
    receipt = dict(status=process.returncode, scope='Controlled local notebook-code execution with real ARM worker/Python network blocked; NOT Kaggle runtime/accuracy.',
                   bundle_manifest_sha256=sha256(bundle / 'bundle-manifest.json'), kaggle_runtime_verified=False,
                   partner_readiness_threshold=None)
    if process.returncode:
        write_json(output / 'validation.json', receipt)
        raise AssertionError('Local notebook failed; primary stdout/stderr/artifacts preserved')
    run = output / 'working/local-notebook-run'
    manifest = json.loads((run / 'manifest.json').read_text())
    submission = json.loads((output / 'working/submission.json').read_text())
    validate_submission(challenges, submission)
    assert manifest['tasks'] == 3 and manifest['queries'] == 4
    assert manifest['learned_queries'] == 2 and manifest['baseline_queries'] == 2
    assert manifest['score']['denominator'] == 4 and manifest['score']['learned_exact'] == 2
    assert manifest['score']['baseline_exact'] == 0
    frozen = json.loads((run / 'all-predictions-frozen.json').read_text())
    for i, row in enumerate(frozen['ledger']):
        assert row['status'] == 'ok' and row['graph']['closed']
        path = run / f'task-{i:05d}/graph.jsonl.gz'
        assert hashlib.sha256(gzip.decompress(path.read_bytes())).hexdigest() == row['graph']['raw_sha256']
    x86 = info['workers']['linux-x86_64-musl']
    x86_info = elf_info(bundle / x86['path'])
    assert x86_info['machine'] == 62 and not x86_info['has_interpreter']
    receipt.update(criterion_met=True, format_valid=True, tasks=3, queries=4, learned_queries=2, baseline_queries=2,
                   controlled_exact=2, denominator=4, native_proof_crc_and_raw_hash_verified=True,
                   x86_elf_verified=True, x86_execution_verified=False,
                   submission_sha256=sha256(output / 'working/submission.json'))
    write_json(output / 'validation.json', receipt)
    return receipt


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    print(json.dumps(validate(args.bundle, args.output), indent=2))
