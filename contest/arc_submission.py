"""Local-only offline ARC adapter. No upload/API/network path; native learning unchanged."""
import argparse
import gzip
import hashlib
import json
import math
import pathlib
import subprocess
import threading
import time

WORKER_TIMEOUT = 30.0
COHORT_BUDGET = 36000.0


def sha256(path):
    h = hashlib.sha256()
    with pathlib.Path(path).open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def write_json(path, obj):
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x', encoding='utf-8') as f:
        json.dump(obj, f, indent=2, allow_nan=False)
        f.write('\n')


def validate_grid(grid, where='grid'):
    if not isinstance(grid, list) or not 1 <= len(grid) <= 30:
        raise ValueError(f'{where}: expected 1..30 rows')
    width = len(grid[0]) if isinstance(grid[0], list) else 0
    if not 1 <= width <= 30:
        raise ValueError(f'{where}: expected 1..30 columns')
    for row in grid:
        if not isinstance(row, list) or len(row) != width:
            raise ValueError(f'{where}: not rectangular')
        if any(type(c) is not int or not 0 <= c <= 9 for c in row):
            raise ValueError(f'{where}: cells must be integer colours 0..9 (not bool)')
    return grid


def load_json_unique(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError(f'Duplicate JSON key: {key}')
            result[key] = value
        return result
    return json.loads(pathlib.Path(path).read_text(), object_pairs_hook=unique)


def load_challenges(path):
    data = load_json_unique(path)
    if not isinstance(data, dict) or not data:
        raise ValueError('Challenges must be a nonempty task dictionary')
    for task, item in data.items():
        if not isinstance(task, str) or not task or not isinstance(item, dict):
            raise ValueError('Invalid task entry')
        train, test = item.get('train'), item.get('test')
        if not isinstance(train, list) or not train or not isinstance(test, list) or not test:
            raise ValueError(f'{task}: nonempty train and test lists required')
        for i, pair in enumerate(train):
            if not isinstance(pair, dict) or 'input' not in pair or 'output' not in pair:
                raise ValueError(f'{task}/train/{i}: missing input/output')
            validate_grid(pair['input'], f'{task}/train/{i}/input')
            validate_grid(pair['output'], f'{task}/train/{i}/output')
        for i, pair in enumerate(test):
            if not isinstance(pair, dict) or 'input' not in pair or 'output' in pair:
                raise ValueError(f'{task}/test/{i}: input only; separate expected outputs')
            validate_grid(pair['input'], f'{task}/test/{i}/input')
    return data


def validate_submission(challenges, submission):
    if not isinstance(submission, dict) or set(submission) != set(challenges):
        raise ValueError('Submission must contain exactly ALL challenge task IDs')
    for task, challenge in challenges.items():
        answers = submission[task]
        if not isinstance(answers, list) or len(answers) != len(challenge['test']):
            raise ValueError(f'{task}: test-output count/order must match test inputs')
        for i, answer in enumerate(answers):
            if not isinstance(answer, dict) or set(answer) != {'attempt_1', 'attempt_2'}:
                raise ValueError(f'{task}/{i}: exactly attempt_1 and attempt_2 required')
            for key in ('attempt_1', 'attempt_2'):
                validate_grid(answer[key], f'{task}/{i}/{key}')
    return True


def wire(grid):
    return '/'.join(','.join(map(str, row)) for row in grid)


def _graph_stream(pipe, path, result):
    raw_hash = hashlib.sha256()
    nodes = 0
    maxref = -1
    roots = 0
    malformed = 0
    faults = 0
    raw_bytes = 0
    try:
        with path.open('wb') as destination, gzip.GzipFile(fileobj=destination, mode='wb', mtime=0) as archive:
            for line in pipe:
                archive.write(line)
                raw_hash.update(line)
                raw_bytes += len(line)
                try:
                    n = json.loads(line)
                    if n['kind'] == 'native-graph-roots':
                        roots += 1
                        refs = n['roots']
                        if not isinstance(refs, list) or not refs:
                            malformed += 1
                            continue
                    elif n['kind'] == 'native-node':
                        if type(n['id']) is not int or n['id'] != nodes:
                            malformed += 1
                        nodes += 1
                        refs = n['children'] + ([n['content']['ref']] if 'ref' in n['content'] else [])
                        faults += 'fault' in n['content']
                    else:
                        malformed += 1
                        continue
                    if any(type(r) is not int or r < 0 for r in refs):
                        malformed += 1
                    else:
                        maxref = max([maxref] + refs)
                except (ValueError, KeyError, TypeError):
                    malformed += 1
        result.update(raw_sha256=raw_hash.hexdigest(), raw_bytes=raw_bytes, nodes=nodes,
                      malformed_lines=malformed, faults=faults,
                      closed=roots == 1 and nodes > maxref and nodes > 0 and malformed == 0 and faults == 0,
                      gzip_sha256=sha256(path), compressed_bytes=path.stat().st_size)
    except Exception as error:
        result.update(closed=False, recording_error=f'{type(error).__name__}: {error}')
    finally:
        pipe.close()


def worker_run(worker, challenge, folder, timeout):
    folder.mkdir()
    bundle = folder / 'input.tsv'
    lines = ['\t'.join(['TRAIN', f'train{i}', wire(p['input']), wire(p['output'])])
             for i, p in enumerate(challenge['train'])]
    lines += ['\t'.join(['QUERY', f'query{i}', wire(p['input'])]) for i, p in enumerate(challenge['test'])]
    bundle.write_text('\n'.join(lines) + '\n')
    graph = {}
    observations = []
    errors = []
    started = time.monotonic()
    status = 'launch-error'
    code = None
    with (folder / 'stdout.jsonl').open('wb') as stdout:
        try:
            process = subprocess.Popen([str(worker), str(bundle), '--cyclic-lattice-actions'],
                                       stdout=stdout, stderr=subprocess.PIPE)
        except OSError as error:
            errors.append(f'{type(error).__name__}: {error}')
        else:
            recorder = threading.Thread(target=_graph_stream, args=(process.stderr, folder / 'graph.jsonl.gz', graph))
            recorder.start()
            try:
                code = process.wait(timeout=timeout)
                status = 'ok' if code == 0 else 'worker-error'
            except subprocess.TimeoutExpired:
                status = 'timeout'
                process.kill()
                code = process.wait()
            finally:
                recorder.join()
    for line in (folder / 'stdout.jsonl').read_text(errors='replace').splitlines():
        try:
            obs = json.loads(line)
            if not isinstance(obs, dict):
                raise ValueError('non-object native observation')
            observations.append(obs)
        except ValueError as error:
            errors.append(f'Invalid native stdout: {error}')
    derivations = [o for o in observations if o.get('kind') == 'derivation']
    if len(derivations) != 1 or derivations[0].get('pairs') != len(challenge['train']):
        errors.append('Native derivation coverage differs from ALL supplied training pairs')
    return dict(status=status, return_code=code, wall_seconds=time.monotonic() - started,
                worker_timeout_seconds=timeout, input_sha256=sha256(bundle),
                stdout_sha256=sha256(folder / 'stdout.jsonl'), graph=graph,
                observations=observations, orchestration_errors=errors)


def choose_predictions(challenge, row):
    predictions, secondaries = {}, {}
    duplicate = False
    malformed_secondary_ids = set()
    malformed_secondary_stream = False
    for o in row.get('observations', []):
        if o.get('kind') == 'prediction':
            key = o.get('id')
            if not isinstance(key, str):
                duplicate = True
                continue
            if key in predictions:
                duplicate = True
            predictions[key] = o
        elif o.get('kind') == 'attempt-2':
            key = o.get('id')
            if not isinstance(key, str):
                malformed_secondary_stream = True
                continue
            if key in secondaries:
                malformed_secondary_ids.add(key)
            secondaries[key] = o
    graph_closed = row.get('graph', {}).get('closed') is True
    evidence_clean = graph_closed and not row.get('orchestration_errors') and not duplicate
    usable = row['status'] == 'ok' and evidence_clean
    expected_ids = {f'query{i}' for i in range(len(challenge['test']))}
    if set(predictions) - expected_ids:
        usable = False
        evidence_clean = False
    if set(secondaries) - expected_ids:
        malformed_secondary_stream = True
    failures = [o for o in row.get('observations', []) if o.get('kind') == 'failure']
    query_failure = (row['status'] == 'worker-error' and len(failures) == 1
                     and failures[0].get('phase') in {'query', 'query-bind'}
                     and failures[0].get('id') in expected_ids)
    secondary_evidence_usable = (evidence_clean and not malformed_secondary_stream
                                 and ((row['status'] == 'ok' and not failures) or query_failure))
    answers, provenance = [], []
    for i, query in enumerate(challenge['test']):
        query_id = f'query{i}'
        pred = predictions.get(query_id, {})
        grid = pred.get('grid')
        reason = row['status']
        native = False
        if usable:
            if pred.get('status') == 'no-matching-valid-relational-encoding':
                reason = 'native-no-match'
            elif 'grid' not in pred:
                reason = 'native-missing-prediction'
            else:
                try:
                    validate_grid(grid, 'native prediction')
                    native = True
                    reason = 'native-returned'
                except ValueError:
                    reason = 'native-invalid-grid'
        elif row['status'] == 'ok':
            reason = 'native-evidence-or-stream-invalid'
        if not native:
            grid = query['input']

        attempt_2 = grid
        attempt_2_origin = 'duplicate-attempt-1'
        attempt_2_reason = 'secondary-absent'
        candidate_origin = None
        candidate_index = None
        candidate_address = None
        all_demonstrations_replayed = False
        independent = False
        secondary = secondaries.get(query_id)
        if malformed_secondary_stream:
            attempt_2_reason = 'secondary-stream-malformed'
        elif query_id in malformed_secondary_ids:
            attempt_2_reason = 'secondary-duplicate-record'
        elif secondary is not None:
            candidate_origin = secondary.get('candidate_origin')
            candidate_index = secondary.get('candidate_index')
            candidate_address = secondary.get('candidate_address')
            all_demonstrations_replayed = secondary.get('all_demonstrations_replayed') is True
            if not secondary_evidence_usable:
                attempt_2_reason = 'secondary-evidence-invalid'
            elif secondary.get('status') != 'distinct-replay-valid-candidate':
                attempt_2_reason = secondary.get('reason', 'secondary-fallback')
            elif (secondary.get('replay_valid') is not True
                  or not all_demonstrations_replayed
                  or secondary.get('independent') is not True
                  or not isinstance(candidate_origin, str)
                  or candidate_origin not in {
                      'prior-replay-valid', 'broad-replay-valid', 'group-replay-valid',
                      'partition-replay-valid', 'cyclic-replay-valid', 'intrinsic-replay-valid'}
                  or type(candidate_index) is not int or candidate_index < 1
                  or candidate_address != f'ordered-replay-valid-candidates/{candidate_index}'):
                attempt_2_reason = 'secondary-provenance-malformed'
            else:
                candidate_grid = secondary.get('grid')
                try:
                    validate_grid(candidate_grid, 'native secondary prediction')
                except ValueError:
                    attempt_2_reason = 'secondary-invalid-grid'
                else:
                    if candidate_grid == grid:
                        attempt_2_reason = 'secondary-output-identical'
                    else:
                        attempt_2 = candidate_grid
                        attempt_2_origin = f'native-{candidate_origin}'
                        attempt_2_reason = 'distinct-replay-valid-candidate'
                        independent = True
        elif not secondary_evidence_usable:
            attempt_2_reason = 'secondary-evidence-invalid'
        answers.append({'attempt_1': [r[:] for r in grid], 'attempt_2': [r[:] for r in attempt_2]})
        provenance.append(dict(query_index=i, learned=native, reason=reason,
                               attempt_1_origin='native' if native else 'baseline-copy-input',
                               attempt_2_origin=attempt_2_origin, attempt_2_reason=attempt_2_reason,
                               secondary_candidate_origin=candidate_origin,
                               secondary_candidate_index=candidate_index,
                               secondary_candidate_address=candidate_address,
                               secondary_all_demonstrations_replayed=all_demonstrations_replayed,
                               secondary_evidence_usable=secondary_evidence_usable,
                               secondary_used_with_primary_fallback=independent and not native,
                               independent_second_attempt=independent))
    return answers, provenance


def score_submission(challenges, submission, provenance, solutions_path):
    # Must only be called AFTER all predictions and statuses have frozen.
    solutions = load_json_unique(solutions_path)
    if not isinstance(solutions, dict) or set(solutions) != set(challenges):
        raise ValueError('Solutions must cover exactly ALL challenge tasks; no filtered denominator')
    per_output = []
    for task, challenge in challenges.items():
        wants = solutions[task]
        if not isinstance(wants, list) or len(wants) != len(challenge['test']):
            raise ValueError(f'{task}: solution output count mismatch')
        for i, want in enumerate(wants):
            validate_grid(want, f'{task}/solution/{i}')
            exact = any(submission[task][i][a] == want for a in ('attempt_1', 'attempt_2'))
            per_output.append(dict(task=task, query_index=i, exact=exact, learned=provenance[task][i]['learned']))
    total = len(per_output)
    correct = sum(r['exact'] for r in per_output)
    return dict(denominator=total, exact=correct, accuracy=correct / total,
                learned_exact=sum(r['exact'] and r['learned'] for r in per_output),
                baseline_exact=sum(r['exact'] and not r['learned'] for r in per_output), per_output=per_output)


def run(challenges_path, worker, output_path, artifacts, *, worker_timeout=WORKER_TIMEOUT,
        cohort_budget=COHORT_BUDGET, solutions_path=None):
    if not all(isinstance(v, (int, float)) and not isinstance(v, bool) and math.isfinite(v) and v > 0
               for v in (worker_timeout, cohort_budget)):
        raise ValueError('Timeout/budget must be finite positive seconds')
    challenges_path, worker = pathlib.Path(challenges_path).resolve(), pathlib.Path(worker).resolve()
    output_path, artifacts = pathlib.Path(output_path).resolve(), pathlib.Path(artifacts).resolve()
    if output_path.exists() or artifacts.exists():
        raise FileExistsError('Refusing existing output/artifact path; use a fresh run directory')
    challenges_hash = sha256(challenges_path)
    challenges = load_challenges(challenges_path)
    if sha256(challenges_path) != challenges_hash:
        raise RuntimeError('Challenges changed while loading')
    worker_hash = sha256(worker)
    artifacts.mkdir(parents=True)
    manifest = dict(scope='LOCAL ONLY; no upload/publication/submission. Format success is NOT score readiness.',
                    challenges_sha256=challenges_hash, worker_sha256=worker_hash,
                    runner_sha256=sha256(__file__), learner='native-generalized-actions', worker_arguments=['--cyclic-lattice-actions'],
                    native_workspace_limit=286720000, worker_timeout_seconds=worker_timeout,
                    cohort_budget_seconds=cohort_budget, budget_scope='worker admission/wait; not total process/RSS/Kaggle deadline guarantee',
                    baseline='copy-input, outside learner',
                    second_attempt='at most one distinct applicable replay-valid candidate; otherwise duplicate attempt 1',
                    partner_readiness_threshold=None, tasks=len(challenges), queries=sum(len(c['test']) for c in challenges.values()))
    write_json(artifacts / 'run-inputs.json', manifest)
    ledger, submission, provenance = [], {}, {}
    started = time.monotonic()
    for index, (task, challenge) in enumerate(challenges.items()):
        remaining = cohort_budget - (time.monotonic() - started)
        if remaining <= 0:
            row = dict(status='budget-unattempted', observations=[], graph={}, orchestration_errors=[])
        else:
            row = worker_run(worker, challenge, artifacts / f'task-{index:05d}', min(worker_timeout, remaining))
        row.update(task=task, train_pairs=len(challenge['train']), query_count=len(challenge['test']))
        answers, origins = choose_predictions(challenge, row)
        submission[task], provenance[task] = answers, origins
        ledger.append(row)
    # This immutable artifact is created before reading optional external solutions.
    write_json(artifacts / 'all-predictions-frozen.json', dict(ledger=ledger, provenance=provenance, submission=submission))
    validate_submission(challenges, submission)
    write_json(output_path, submission)
    manifest.update(format_valid=True, submission_sha256=sha256(output_path),
                    freeze_sha256=sha256(artifacts / 'all-predictions-frozen.json'),
                    learned_queries=sum(r['learned'] for v in provenance.values() for r in v),
                    baseline_queries=sum(not r['learned'] for v in provenance.values() for r in v),
                    independent_second_attempts=sum(r['independent_second_attempt'] for v in provenance.values() for r in v),
                    duplicate_second_attempts=sum(not r['independent_second_attempt'] for v in provenance.values() for r in v),
                    task_status_counts={s: sum(r['status'] == s for r in ledger) for s in sorted({r['status'] for r in ledger})},
                    wall_seconds=time.monotonic() - started)
    if sha256(worker) != worker_hash or sha256(challenges_path) != manifest['challenges_sha256']:
        manifest['integrity_error'] = 'Worker/challenges changed during run'
        write_json(artifacts / 'manifest.json', manifest)
        raise RuntimeError(manifest['integrity_error'])
    if solutions_path is not None:
        try:
            score = score_submission(challenges, submission, provenance, solutions_path)
            write_json(artifacts / 'scores.json', score)
            manifest.update(solutions_sha256=sha256(solutions_path), score=score)
        except Exception as error:
            manifest['scoring_error'] = f'{type(error).__name__}: {error}'
            write_json(artifacts / 'manifest.json', manifest)
            raise
    write_json(artifacts / 'manifest.json', manifest)
    return manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--challenges', required=True)
    parser.add_argument('--worker', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--artifacts', required=True)
    parser.add_argument('--solutions')
    parser.add_argument('--worker-timeout-seconds', type=float, default=WORKER_TIMEOUT)
    parser.add_argument('--cohort-budget-seconds', type=float, default=COHORT_BUDGET)
    args = parser.parse_args()
    result = run(args.challenges, args.worker, args.output, args.artifacts,
                 worker_timeout=args.worker_timeout_seconds, cohort_budget=args.cohort_budget_seconds,
                 solutions_path=args.solutions)
    print(json.dumps({k: result[k] for k in ('format_valid', 'tasks', 'queries', 'learned_queries',
                                              'baseline_queries', 'independent_second_attempts',
                                              'duplicate_second_attempts', 'partner_readiness_threshold')}, indent=2))


if __name__ == '__main__':
    main()
