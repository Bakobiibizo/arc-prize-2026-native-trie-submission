"""Local pinned notebook replay on the historical 50-task cohort. No upload/submission.
All predictions freeze before expected test fields are read; not independent validation.
"""
import argparse
import gzip
import hashlib
import json
import os
import pathlib
import signal
import subprocess
import sys
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'contest'))
from arc_submission import sha256, write_json, validate_submission, score_submission
from notebook_support import verify_bundle


def run(output):
    output = pathlib.Path(output).resolve()
    if output.exists():
        raise FileExistsError('Preserve previous run; choose a new output directory')
    bundle = ROOT / 'evidence/contest-local-setup-001/assets/native-arc-local'
    bundle_manifest = verify_bundle(bundle)
    pool_path = ROOT / 'evidence/source-groups-full-pool-001/pool.json'
    pool = json.loads(pool_path.read_text())
    archive_path = ROOT / 'evidence/arc-color-development-001/source.zip'
    assert sha256(archive_path) == 'c3f9f6975d8432133419fd0e1677c42289666517af4e6f83d8a291690e14d49f'
    assert len(pool['selected']) == 50 and len(set(pool['selected'])) == 50
    output.mkdir(parents=True)
    archive = zipfile.ZipFile(archive_path)
    challenges = {}
    members = {}
    for task in pool['selected']:
        info = pool['inputs'][task]
        raw = archive.read(info['source'])
        assert hashlib.sha256(raw).hexdigest() == info['source_sha256']
        data = json.loads(raw)
        # Deliberately do not access ANY test output fields here.
        challenges[task] = dict(train=data['train'], test=[dict(input=q['input']) for q in data['test']])
        members[task] = dict(source=info['source'], sha256=info['source_sha256'], train_pairs=len(data['train']), queries=len(data['test']))
    assert sum(len(d['test']) for d in challenges.values()) == 51
    write_json(output / 'challenges.json', challenges)
    config = dict(bundle=str(bundle), challenges=str(output / 'challenges.json'), work=str(output / 'working'),
                  run_name='historical-local-run', solutions=None, worker_timeout_seconds=30,
                  cohort_budget_seconds=300)
    write_json(output / 'protocol.json', dict(origin='Operator asks run of LOCAL setup, no upload/publication/submission',
               scope='Exact historical50/all51/allpairs; not independent accuracy/leaderboard',
               source_revision=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
               script_sha256=sha256(__file__), pool_sha256=sha256(pool_path), members=members,
               bundle_manifest_sha256=sha256(bundle / 'bundle-manifest.json'), bundle_source_revision=bundle_manifest['source_revision'],
               worker_timeout_seconds=30, cohort_budget_seconds=300,
               limit_basis='Same prior50-task development worker/cohort envelope; not default10h/full12h guarantee',
               notebook_supervisor_limit_seconds=390, partner_readiness_threshold=None,
               kaggle_runtime_verified=False, source_archive_sha256=sha256(archive_path)))
    script = output / 'execute-notebook.py'
    script.write_text('''import json,pathlib,socket
def denied(*args,**kwargs):raise RuntimeError('Network forbidden in LOCAL notebook replay')
socket.socket=denied
socket.create_connection=denied
CONFIG=''' + repr(config) + '''
nb=json.loads((pathlib.Path(CONFIG['bundle'])/'contest/offline_submission.ipynb').read_text())
namespace={'CONFIG':CONFIG}
for cell in nb['cells']:
 if cell['cell_type']=='code':exec(compile(''.join(cell['source']),cell['id'],'exec'),namespace)
''')
    timeout = False
    with (output / 'notebook.stdout').open('wb') as so, (output / 'notebook.stderr').open('wb') as se:
        process = subprocess.Popen([sys.executable, str(script)], cwd=output, stdout=so, stderr=se, start_new_session=True)
        try:
            code = process.wait(timeout=390)
        except subprocess.TimeoutExpired:
            timeout = True
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait()
    write_json(output / 'notebook-status.json', dict(return_code=code, supervisor_timeout=timeout))
    if code != 0:
        raise RuntimeError('Notebook failed; primary status/stdout/stderr/partial artifacts preserved, no retry')
    working = output / 'working/historical-local-run'
    frozen_path = working / 'all-predictions-frozen.json'
    frozen_hash = sha256(frozen_path)
    frozen = json.loads(frozen_path.read_text())
    manifest = json.loads((working / 'manifest.json').read_text())
    submission = json.loads((output / 'working/submission.json').read_text())
    validate_submission(challenges, submission)
    assert manifest['tasks'] == 50 and manifest['queries'] == 51
    # Freeze exists: only NOW read expected outputs, including the full denominator.
    solutions = {}
    for task in pool['selected']:
        data = json.loads(archive.read(pool['inputs'][task]['source']))
        solutions[task] = [q['output'] for q in data['test']]
    write_json(output / 'solutions-after-freeze.json', solutions)
    score = score_submission(challenges, submission, frozen['provenance'], output / 'solutions-after-freeze.json')
    write_json(output / 'scores.json', score)
    prior = json.loads((ROOT / 'evidence/source-groups-full-001/ledger.json').read_text())
    prior_by_id = {r['task']:r for r in prior}
    comparisons = []
    for i, row in enumerate(frozen['ledger']):
        before = [r for r in prior_by_id[row['task']]['observations'] if r['kind'] == 'prediction']
        after = [r for r in row.get('observations', []) if r['kind'] == 'prediction']
        comparisons.append(dict(task=row['task'], unchanged=before == after))
        if row['status'] != 'budget-unattempted':
            p = working / f'task-{i:05d}/graph.jsonl.gz'
            assert hashlib.sha256(gzip.decompress(p.read_bytes())).hexdigest() == row['graph']['raw_sha256']
    assert sha256(frozen_path) == frozen_hash
    result = dict(scope='LOCAL historical development replay, not fresh validation/leaderboard/partner readiness',
                  format_valid=True, tasks=50, queries=51, train_pairs=sum(m['train_pairs'] for m in members.values()),
                  learned_queries=manifest['learned_queries'], baseline_queries=manifest['baseline_queries'],
                  task_status_counts=manifest['task_status_counts'], score={k:v for k,v in score.items() if k!='per_output'},
                  source_prediction_comparisons=comparisons, prior_native_predictions_unchanged=all(r['unchanged'] for r in comparisons),
                  source_proof_crc_raw_sha_verified=True, freeze_sha256=frozen_hash,
                  submission_sha256=sha256(output / 'working/submission.json'), partner_readiness_threshold=None,
                  kaggle_runtime_verified=False, uploaded=False, published=False, submitted=False)
    write_json(output / 'result.json', result)
    print(json.dumps({k:v for k,v in result.items() if k!='source_prediction_comparisons'}, indent=2))
    assert len(frozen['ledger']) == 50 and score['denominator'] == 51
    assert result['prior_native_predictions_unchanged'], 'Native outcomes changed; preserve evidence and diagnose'
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    run(args.output)
