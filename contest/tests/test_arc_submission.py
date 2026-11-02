"""Orchestration mocks only; separate recorded Rust/notebook integration checks required."""
import copy
import json
import pathlib
import socket
import sys
import tempfile
import unittest
from unittest import mock

CONTEST = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(CONTEST))
import arc_submission as arc
import notebook_support as support


class AdapterTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.data = {
            '../unsafe-id': {'train': [{'input': [[2]], 'output': [[5]]}, {'input': [[2, 2]], 'output': [[5, 5]]}],
                             'test': [{'input': [[2]]}, {'input': [[3, 0]]}]},
            'other': {'train': [{'input': [[1]], 'output': [[1]]}], 'test': [{'input': [[1]]}]},
        }
        self.challenges = self.root / 'challenges.json'
        self.challenges.write_text(json.dumps(self.data))

    def fake_worker(self, behavior='normal'):
        path = self.root / ('worker-' + behavior)
        graph = [{'kind': 'native-graph-roots', 'roots': [0]},
                 {'kind': 'native-node', 'id': 0, 'token_hex': '00', 'children': [], 'content': {'end': True}}]
        script = f'''#!{sys.executable}
import json,sys,time
lines=[s.split('\\t') for s in open(sys.argv[1]).read().splitlines()]
train=[r for r in lines if r[0]=='TRAIN'];query=[r for r in lines if r[0]=='QUERY']
behavior={behavior!r}
if behavior=='generalized-routing' and sys.argv[2:] != ['--cyclic-lattice-actions']:sys.exit(23)
if behavior=='timeout':time.sleep(1)
for row in {graph!r}:
 if behavior=='badgraph' and row.get('kind')=='native-node':row['children']=[99]
 print(json.dumps(row),file=sys.stderr)
if behavior=='failed':
 print(json.dumps(dict(kind='failure',phase='teach',error='WorkspaceLimit')));sys.exit(1)
print(json.dumps(dict(kind='derivation',pairs=len(train),matched=True)))
for r in query:
 if behavior=='missing' and r[1]=='query1':continue
 if behavior in ('no-match','primary-inapplicable'):o=dict(kind='prediction',id=r[1],status='no-matching-valid-relational-encoding')
 elif behavior=='primary-fallback':o=None
 elif behavior=='badgrid':o=dict(kind='prediction',id=r[1],grid=[[True]])
 elif behavior=='badid':o=dict(kind='prediction',id=['bad'],grid=[[9]])
 else:o=dict(kind='prediction',id=r[1],grid=[[9]])
 if o is not None:print(json.dumps(o))
 if behavior=='duplicate':print(json.dumps(o))
 if behavior in ('distinct-secondary','identical-secondary','malformed-secondary','duplicate-secondary','bad-secondary-provenance','primary-inapplicable','primary-fallback'):
  second=dict(kind='attempt-2',id=r[1],status='distinct-replay-valid-candidate',reason='secondary-applicable',candidate_origin='cyclic-replay-valid',candidate_index=1,candidate_address='ordered-replay-valid-candidates/1',all_demonstrations_replayed=True,replay_valid=True,independent=True,grid=[[8 if behavior in ('distinct-secondary','bad-secondary-provenance','primary-inapplicable','primary-fallback') else 9]])
  if behavior=='malformed-secondary':second['grid']=[[True]]
  if behavior=='bad-secondary-provenance':second['all_demonstrations_replayed']=False
  print(json.dumps(second))
  if behavior=='duplicate-secondary':print(json.dumps(second))
 if behavior=='primary-fallback':
  print(json.dumps(dict(kind='failure',phase='query',id=r[1],error='NotReady')))
  sys.exit(1)
'''
        path.write_text(script)
        path.chmod(0o700)
        return path

    def execute(self, behavior='normal', solutions=None, **options):
        return arc.run(self.challenges, self.fake_worker(behavior), self.root / 'output/submission.json',
                       self.root / 'evidence', solutions_path=solutions, **options)

    def test_all_tasks_multi_query_order_and_exact_two_attempts(self):
        with mock.patch.object(socket, 'socket', side_effect=AssertionError('network forbidden')):
            result = self.execute()
        output = json.loads((self.root / 'output/submission.json').read_text())
        self.assertTrue(arc.validate_submission(self.data, output))
        self.assertEqual(list(output), list(self.data))
        self.assertEqual(len(output['../unsafe-id']), 2)
        self.assertEqual(output['../unsafe-id'][0]['attempt_1'], [[9]])
        self.assertEqual(output['../unsafe-id'][0]['attempt_2'], [[9]])
        self.assertEqual(result['learned_queries'], 3)
        self.assertIsNone(result['partner_readiness_threshold'])
        wire = (self.root / 'evidence/task-00000/input.tsv').read_text()
        self.assertEqual(wire.count('TRAIN\t'), 2)
        self.assertNotIn('../unsafe-id', wire)
        self.assertTrue((self.root / 'evidence/task-00000/graph.jsonl.gz').is_file())

    def test_worker_uses_generalized_benchmark_entrypoint(self):
        result = self.execute('generalized-routing')
        self.assertEqual(result['task_status_counts'], {'ok': 2})
        self.assertEqual(result['learned_queries'], 3)
        self.assertEqual(result['worker_arguments'], ['--cyclic-lattice-actions'])
        self.assertEqual(result['learner'], 'native-generalized-actions')

    def test_distinct_secondary_is_attempt_2_with_explicit_provenance(self):
        result = self.execute('distinct-secondary')
        output = json.loads((self.root / 'output/submission.json').read_text())
        frozen = json.loads((self.root / 'evidence/all-predictions-frozen.json').read_text())
        self.assertEqual(output['other'][0], {'attempt_1': [[9]], 'attempt_2': [[8]]})
        self.assertEqual(result['independent_second_attempts'], 3)
        self.assertEqual(result['duplicate_second_attempts'], 0)
        self.assertEqual(frozen['provenance']['other'][0]['attempt_2_origin'],
                         'native-cyclic-replay-valid')
        self.assertTrue(frozen['provenance']['other'][0]['independent_second_attempt'])

    def test_secondary_survives_primary_inapplicability_and_application_fallback(self):
        for behavior, worker_status in (
            ('primary-inapplicable', 'ok'),
            ('primary-fallback', 'worker-error'),
        ):
            with self.subTest(behavior=behavior):
                result = arc.run(self.challenges, self.fake_worker(behavior),
                                 self.root / behavior / 'submission.json', self.root / behavior / 'evidence')
                output = json.loads((self.root / behavior / 'submission.json').read_text())
                frozen = json.loads((self.root / behavior / 'evidence/all-predictions-frozen.json').read_text())
                self.assertEqual(output['other'][0], {'attempt_1': [[1]], 'attempt_2': [[8]]})
                provenance = frozen['provenance']['other'][0]
                self.assertEqual(provenance['attempt_1_origin'], 'baseline-copy-input')
                self.assertEqual(provenance['attempt_2_origin'], 'native-cyclic-replay-valid')
                self.assertTrue(provenance['secondary_all_demonstrations_replayed'])
                self.assertTrue(provenance['secondary_evidence_usable'])
                self.assertTrue(provenance['secondary_used_with_primary_fallback'])
                self.assertEqual(result['task_status_counts'][worker_status], 2)

    def test_secondary_identical_malformed_duplicate_or_absent_falls_back_honestly(self):
        expectations = {
            'identical-secondary': 'secondary-output-identical',
            'malformed-secondary': 'secondary-invalid-grid',
            'duplicate-secondary': 'secondary-duplicate-record',
            'bad-secondary-provenance': 'secondary-provenance-malformed',
            'normal': 'secondary-absent',
        }
        for behavior, reason in expectations.items():
            with self.subTest(behavior=behavior):
                result = arc.run(self.challenges, self.fake_worker(behavior),
                                 self.root / behavior / 'submission.json', self.root / behavior / 'evidence')
                output = json.loads((self.root / behavior / 'submission.json').read_text())
                frozen = json.loads((self.root / behavior / 'evidence/all-predictions-frozen.json').read_text())
                self.assertEqual(output['other'][0]['attempt_2'], output['other'][0]['attempt_1'])
                self.assertEqual(frozen['provenance']['other'][0]['attempt_2_reason'], reason)
                self.assertEqual(result['independent_second_attempts'], 0)
                self.assertEqual(result['duplicate_second_attempts'], 3)

    def test_absent_one_prediction_does_not_discard_other_valid_predictions(self):
        result = self.execute('missing')
        self.assertEqual((result['learned_queries'], result['baseline_queries']), (2, 1))
        frozen = json.loads((self.root / 'evidence/all-predictions-frozen.json').read_text())
        self.assertEqual(frozen['provenance']['../unsafe-id'][1]['reason'], 'native-missing-prediction')
        self.assertEqual(frozen['submission']['../unsafe-id'][1]['attempt_1'], [[3, 0]])

    def test_no_match_is_explicit_baseline_not_learned(self):
        result = self.execute('no-match')
        self.assertEqual((result['learned_queries'], result['baseline_queries']), (0, 3))
        frozen = json.loads((self.root / 'evidence/all-predictions-frozen.json').read_text())
        self.assertEqual(frozen['provenance']['other'][0]['reason'], 'native-no-match')

    def test_failed_worker_even_with_closed_graph_is_not_native_success(self):
        result = self.execute('failed')
        self.assertEqual(result['task_status_counts'], {'worker-error': 2})
        self.assertEqual(result['learned_queries'], 0)
        frozen = json.loads((self.root / 'evidence/all-predictions-frozen.json').read_text())
        self.assertTrue(frozen['ledger'][0]['graph']['closed'])
        self.assertEqual(frozen['ledger'][0]['return_code'], 1)

    def test_timeout_no_retry_preserves_status(self):
        result = self.execute('timeout', worker_timeout=0.02)
        self.assertEqual(result['task_status_counts'], {'timeout': 2})
        self.assertEqual(result['baseline_queries'], 3)

    def test_budget_unattempted_preserves_full_denominator(self):
        result = self.execute(cohort_budget=1e-20)
        self.assertEqual(result['task_status_counts'], {'budget-unattempted': 2})
        self.assertEqual(result['baseline_queries'], 3)
        self.assertEqual(result['queries'], 3)

    def test_bad_graph_duplicate_ids_bad_id_and_bad_grid_cannot_be_learned(self):
        for behavior in ('badgraph', 'duplicate', 'badid', 'badgrid'):
            with self.subTest(behavior=behavior):
                result = arc.run(self.challenges, self.fake_worker(behavior), self.root / behavior / 'submission.json',
                                 self.root / behavior / 'evidence')
                self.assertEqual(result['learned_queries'], 0)
                self.assertEqual(result['baseline_queries'], 3)

    def test_separate_scoring_is_after_freeze_and_counts_every_output(self):
        solutions = self.root / 'solutions.json'
        solutions.write_text(json.dumps({'../unsafe-id': [[[9]], [[3, 0]]], 'other': [[[8]]]}))
        original = arc.score_submission
        def checked(*args):
            self.assertTrue((self.root / 'evidence/all-predictions-frozen.json').is_file())
            self.assertTrue((self.root / 'output/submission.json').is_file())
            return original(*args)
        with mock.patch.object(arc, 'score_submission', side_effect=checked):
            result = self.execute('missing', solutions=solutions)
        self.assertEqual(result['score']['denominator'], 3)
        self.assertEqual(result['score']['exact'], 2)
        self.assertEqual((result['score']['learned_exact'], result['score']['baseline_exact']), (1, 1))

    def test_bad_solution_coverage_fails_scoring_without_erasing_outputs(self):
        solutions = self.root / 'solutions.json'
        solutions.write_text('{}')
        with self.assertRaises(ValueError):
            self.execute(solutions=solutions)
        self.assertTrue((self.root / 'output/submission.json').is_file())
        self.assertIn('scoring_error', json.loads((self.root / 'evidence/manifest.json').read_text()))

    def test_duplicate_json_task_keys_are_not_silently_dropped(self):
        item = json.dumps(self.data['other'])
        self.challenges.write_text('{"same":' + item + ',"same":' + item + '}')
        with self.assertRaises(ValueError):
            self.execute()

    def test_challenge_gold_is_rejected_before_worker(self):
        self.data['other']['test'][0]['output'] = [[1]]
        self.challenges.write_text(json.dumps(self.data))
        with self.assertRaises(ValueError):
            self.execute()
        self.assertFalse((self.root / 'evidence').exists())

    def test_refuses_overwrite_and_invalid_limits(self):
        self.execute()
        digest = arc.sha256(self.root / 'output/submission.json')
        with self.assertRaises(FileExistsError):
            self.execute()
        self.assertEqual(digest, arc.sha256(self.root / 'output/submission.json'))
        for invalid in (0, -1, float('inf'), float('nan'), True):
            with self.assertRaises(ValueError):
                arc.run(self.challenges, self.root / 'not-used', self.root / 'other-output',
                        self.root / 'other-evidence', worker_timeout=invalid)

    def test_validator_rejects_missing_tasks_attempts_queries_and_bad_grids(self):
        valid = {k: [{'attempt_1': p['input'], 'attempt_2': p['input']} for p in v['test']] for k, v in self.data.items()}
        changes = []
        x = copy.deepcopy(valid); del x['other']; changes.append(x)
        x = copy.deepcopy(valid); x['other'].append(x['other'][0]); changes.append(x)
        x = copy.deepcopy(valid); del x['other'][0]['attempt_2']; changes.append(x)
        for bad_grid in ([], [[True]], [[10]], [[1.0]], [[1], [1, 2]], [[0] * 31]):
            x = copy.deepcopy(valid); x['other'][0]['attempt_1'] = bad_grid; changes.append(x)
        for x in changes:
            with self.assertRaises(ValueError):
                arc.validate_submission(self.data, x)

    def test_unsupported_architecture_and_tampered_bundle_fail_closed(self):
        bundle = self.root / 'bundle'; bundle.mkdir()
        (bundle / 'runner').write_text('original')
        (bundle / 'bundle-manifest.json').write_text(json.dumps({'files': {'runner': arc.sha256(bundle / 'runner')}, 'workers': {}}))
        with self.assertRaises(RuntimeError):
            support.prepare_worker(bundle, self.root / 'work', machine='riscv64')
        (bundle / 'runner').write_text('tampered')
        with self.assertRaises(ValueError):
            support.verify_bundle(bundle)

    def test_notebook_is_unexecuted_and_code_cells_compile(self):
        notebook = json.loads((CONTEST / 'offline_submission.ipynb').read_text())
        self.assertEqual(notebook['nbformat'], 4)
        for cell in notebook['cells']:
            if cell['cell_type'] == 'code':
                self.assertEqual(cell['outputs'], [])
                self.assertIsNone(cell['execution_count'])
                compile(''.join(cell['source']), cell['id'], 'exec')


if __name__ == '__main__':
    unittest.main()
