"""Bounded setup, exact-pair, transfer, and row-retention mutations; no ARM run."""
import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
import yaml

from proposal_guard import check
from transfer import ALLOWED, record, verify
from semantics import canonical_graph, sha
from build_arm import extract_artifact, graph_probe_cfg
from measure import capture_two_arms, complete_sample_row
from report import report
from napi_parity import adapter_lock_notice_count
from common import ExperimentFailure

HERE = Path(__file__).resolve().parent
OLD = Path('/tmp/nm2022-v4-corrected-proposal/.github/scripts/repeated-target-native-1971')


class ProposalTests(unittest.TestCase):
    def test_native_adapter_allows_only_exact_retained_lock_notice_lines(self):
        self.assertEqual(adapter_lock_notice_count(b''), 0)
        self.assertEqual(adapter_lock_notice_count(
            b'waiting for lock held by pid 123 for 0s\n'
            b'waiting for lock held by another no-mistakes invocation for 0s\n'), 2)
        for changed in (b'warning: waiting for lock held by pid 123 for 0s\n',
                        b'waiting for lock held by pid 123 for 1s\n',
                        b'waiting for lock held by pid 123 for 0s\nother failure\n',
                        b'\xff'):
            with self.subTest(changed=changed), self.assertRaises(ExperimentFailure):
                adapter_lock_notice_count(changed)

    def test_missing_native_facade_pins_reject_preflight_before_source_or_build_work(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        spec['status'] = 'REVIEWED AND FROZEN'
        invalid = (None, {}, {'index.js': 'a' * 64},
                   {'index.js': None, 'index.mjs': 'b' * 64},
                   {'index.js': 'g' * 64, 'index.mjs': 'b' * 64},
                   {'index.js': 'a' * 63, 'index.mjs': 'b' * 64})
        for pins in invalid:
            with self.subTest(pins=pins), patch('proposal_guard.git') as lookup:
                spec['semantic_policy']['napi_facade_sha256'] = pins
                with self.assertRaisesRegex(AssertionError, 'facade pins'):
                    check(spec, Path('/missing-source'), 'pair-preflight')
                lookup.assert_not_called()

    def test_original_sixteen_workloads_retained_exactly(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        old = json.loads((OLD / 'pair.json').read_text())
        self.assertEqual(spec['workloads'], old['workloads'])
        self.assertEqual(spec['workload_count'], 16)
        self.assertEqual(spec['sampling']['long_orders'], ['ABBA', 'BAAB'])
        self.assertEqual(spec['sampling']['long_blocks_per_order'], 4)
        self.assertEqual(spec['sampling']['metrics'], ['cpu_s', 'user_s', 'system_s', 'wall_s', 'rss_kib'])

    def test_push_registers_only_and_every_runtime_checkout_is_full(self):
        workflow = yaml.safe_load((HERE / 'workflow-proposal.yml').read_text())
        jobs = workflow['jobs']
        self.assertEqual(jobs['registration']['if'], "${{ github.event_name == 'push' }}")
        self.assertEqual(jobs['pair-preflight']['if'], "${{ github.event_name == 'workflow_dispatch' }}")
        self.assertEqual(jobs['production-build']['needs'], 'pair-preflight')
        self.assertEqual(jobs['layout-build']['needs'], 'pair-preflight')
        self.assertEqual(jobs['measurement']['if'], "${{ github.event_name == 'workflow_dispatch' && always() }}")
        self.assertEqual(set(jobs['measurement']['needs']), {'pair-preflight', 'production-build', 'layout-build'})
        for name in ('pair-preflight', 'production-build', 'layout-build', 'measurement'):
            checkout = [step for step in jobs[name]['steps'] if step.get('uses', '').startswith('actions/checkout@')]
            self.assertEqual(len(checkout), 1)
            self.assertEqual(checkout[0]['with']['fetch-depth'], 0)

    def test_workflow_timeouts_match_reviewed_pair_budgets(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        jobs = yaml.safe_load((HERE / 'workflow-proposal.yml').read_text())['jobs']
        budgets = spec['budgets_minutes']
        self.assertEqual(jobs['measurement']['timeout-minutes'], budgets['measurement_job'])
        steps = jobs['measurement']['steps']
        for stage, budget in (('semantic-preflight', 'semantic_preflight'),
                              ('napi-parity', 'napi_parity'),
                              ('sample', 'sample'), ('summary', 'summary')):
            selected = [step for step in steps if 'stage.py ' + stage in step.get('run', '')]
            self.assertEqual(len(selected), 1, stage)
            self.assertEqual(selected[0]['timeout-minutes'], budgets[budget], stage)

    def test_baseline_graph_probe_keeps_legacy_compile_shape(self):
        self.assertIsNone(graph_probe_cfg('baseline'))
        self.assertEqual(graph_probe_cfg('candidate'), 'alternatives_api')
        source = (HERE / 'graph-probe.rs').read_text()
        self.assertIn('#[cfg(not(alternatives_api))]', source)
        self.assertIn('#[cfg(alternatives_api)]', source)

    def test_exact_pair_identity_and_draft_rejects_every_stage(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        self.assertEqual(spec['baseline']['revision'], '31a0d460d81616146351a2d0c5bff3ae3b625d2f')
        self.assertEqual(spec['candidate']['revision'], 'bb5328e2460f49713ee13efe466d55fdd25f18a4')
        self.assertEqual(spec['candidate']['merge_parents'],
                         [spec['baseline']['revision'], spec['pr']['head_sha']])
        self.assertEqual(spec['pr']['required_successful_ci_run'], 38836020089)
        self.assertEqual(spec['pr']['head_sha'], '714126d7d3a4be1f0fec97f6b0c84270bbadb18b')
        self.assertEqual(spec['pr']['four_codecov_check_runs'], {
            'codecov/patch/rust': 116767823583,
            'codecov/project/rust': 116767819194,
            'codecov/patch/javascript': 116766658469,
            'codecov/project/javascript': 116766650663,
        })
        self.assertEqual(spec['status'], 'REVIEWED AND FROZEN')
        # Draft metadata must remain blocked at every stage after promotion.
        spec['status'] = 'DRAFT SOURCE ONLY'
        for stage in ('pair-preflight', 'production-build', 'layout-build',
                      'semantic-preflight', 'napi-parity', 'sample', 'summary'):
            with self.subTest(stage=stage), self.assertRaisesRegex(AssertionError, 'draft experiment'):
                check(spec, Path('/missing-source'), stage)

    def test_exact_overlay_requires_external_sha_and_synthetic_merge_parents(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        baseline, source, merge, driver = ('a' * 40, 'b' * 40, 'c' * 40, 'd' * 40)
        spec.update(status='REVIEWED AND FROZEN', inputs={},
                    source_git_inventory_sha256={'baseline': 'e' * 64, 'candidate': 'f' * 64})
        spec['baseline'].update(revision=baseline, tree='1' * 40)
        spec['candidate'].update(revision=merge, proposed_revision=merge, tree='2' * 40, pr_head=source)
        spec['pr'].update(head_sha=source, required_successful_ci_run=1,
                          four_codecov_check_runs={name: i for i, name in enumerate(
                              ('codecov/patch/rust', 'codecov/project/rust',
                               'codecov/patch/javascript', 'codecov/project/javascript'), 1)})
        spec['diagnostic_parent'] = source
        spec['semantic_policy'].update(status='REVIEWED FULL REPORTS', report_hashes={'graph': {'a': 'x'}})
        def fake_git(_repo, *args):
            if args == ('rev-parse', merge + '^{tree}'):
                return '2' * 40
            if args == ('rev-parse', baseline + '^{tree}'):
                return '1' * 40
            if args == ('show', '-s', '--format=%P', merge):
                return baseline + ' ' + source
            if args == ('show', '-s', '--format=%P', driver):
                return source
            if args[0] == 'merge-base':
                return baseline
            if args[0] == 'diff':
                return 'A\t.github/scripts/repeated-target-native-1971-next/stage.py'
            raise AssertionError(args)
        with patch('proposal_guard.git', side_effect=fake_git):
            with self.assertRaises(AssertionError):
                check(spec, Path('/unused'), 'pair-preflight', driver, None)
            self.assertTrue(check(spec, Path('/unused'), 'pair-preflight', driver, driver))
            spec['diagnostic_parent'] = merge
            with self.assertRaises(AssertionError):
                check(spec, Path('/unused'), 'pair-preflight', driver, driver)

    def test_private_transfer_rejects_changed_bytes_and_extra_directory(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for file in ALLOWED:
                (root / file).write_bytes(file.encode())
            manifest = record(root, 'candidate', 'a' * 40, 'b' * 40, 'c' * 64,
                              'rustc 1.99.1', ['default', 'mermaid-validation'],
                              {'opt_level': 3, 'lto': True, 'codegen_units': 1}, 'alternatives_api')
            expected = {key: manifest[key] for key in ('arm', 'revision', 'tree',
                         'source_inventory_sha256', 'compiler', 'features', 'profile', 'graph_probe_cfg')}
            self.assertTrue(verify(root, manifest, expected))
            (root / 'cli').write_bytes(b'wrong')
            with self.assertRaises(AssertionError):
                verify(root, manifest, expected)
            (root / 'cli').write_bytes(b'cli')
            (root / 'source').mkdir()
            (root / 'source/private.rs').write_text('private')
            with self.assertRaises(AssertionError):
                record(root, 'candidate', 'a' * 40, 'b' * 40, 'c' * 64,
                       'rustc 1.99.1', ['default', 'mermaid-validation'], {'lto': True}, 'alternatives_api')

    def test_full_graph_hash_rejects_target_and_callable_mutations(self):
        graph = {'semantic': {'sites': [{'known': [{'name': 'transaction.write',
                  'node': '["symbol","x","write","Some(CallableId(9))"]'}],
                  'callee': 'query', 'file': 'entry.mts', 'offset': 42, 'unknown': True}],
                  'nodes': ['["symbol","x","write","Some(CallableId(9))"]'],
                  'edges': [], 'importer_view': []}}
        baseline = sha(canonical_graph(json.loads(json.dumps(graph))))
        changed = json.loads(json.dumps(graph))
        changed['semantic']['sites'][0]['known'][0]['name'] = 'other.write'
        self.assertNotEqual(sha(canonical_graph(changed)), baseline)
        changed = json.loads(json.dumps(graph))
        changed['semantic']['nodes'][0] = changed['semantic']['nodes'][0].replace('CallableId(9)', 'CallableId(10)')
        self.assertNotEqual(sha(canonical_graph(changed)), baseline)

    def test_compiler_artifact_requires_owned_fresh_default_build(self):
        source = Path('/tmp/no-mistakes/src/main.rs')
        artifact = {'reason': 'compiler-artifact', 'target': {'name': 'no-mistakes',
                    'kind': ['bin'], 'src_path': str(source)}, 'fresh': False,
                    'features': ['default', 'mermaid-validation'],
                    'profile': {'opt_level': '3', 'debuginfo': 0, 'debug_assertions': False,
                                'overflow_checks': False, 'test': False},
                    'executable': '/tmp/no-mistakes/target/release/no-mistakes'}
        self.assertEqual(extract_artifact((json.dumps(artifact) + '\n').encode(), 'bin', source), artifact)
        artifact['fresh'] = True
        with self.assertRaises(Exception):
            extract_artifact((json.dumps(artifact) + '\n').encode(), 'bin', source)

    def test_second_arm_startup_and_calibration_failure_retains_first_row(self):
        for phase in ('startup', 'calibration'):
            with self.subTest(phase=phase), tempfile.TemporaryDirectory() as name:
                root = Path(name)
                def fake_sample(_root, _workload, arm, _count, _affinity):
                    if arm == 'candidate':
                        raise RuntimeError('injected second-arm failure')
                    return {'arm': arm, 'valid': True, 'cpu_s': .125, 'wall_s': .15, 'rss_kib': 100}
                with patch('measure.sample', side_effect=fake_sample):
                    with self.assertRaisesRegex(RuntimeError, 'injected second-arm failure'):
                        capture_two_arms(root, {'name': 'physical'}, phase, 1, [0, 1, 2, 3])
                self.assertEqual(json.loads((root / f'public/physical-{phase}.json').read_text()),
                                 [{'arm': 'baseline', 'valid': True, 'cpu_s': .125, 'wall_s': .15, 'rss_kib': 100}])

    def test_output_failure_keeps_wait4_resources(self):
        usage = SimpleNamespace(ru_utime=.4, ru_stime=.1, ru_maxrss=321)
        with patch('measure.inspect_sample', side_effect=OSError('injected unreadable output')):
            row = complete_sample_row(Path('/tmp'), Path('/tmp/missing'), {'name': 'physical'},
                                      'candidate', 4, 0, False, usage, .7, [0, 1, 2, 3])
        self.assertFalse(row['valid'])
        self.assertEqual(row['error_code'], 'sample_output_inspection_failed')
        self.assertEqual((row['cpu_s'], row['wall_s'], row['rss_kib']), (.5, .7, 321))

    def test_final_ledger_keeps_remote_and_upstream_failures(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            remote = {'eligible': False, 'status': 'historical or unverified',
                      'error_code': 'candidate_remote_head_changed'}
            with patch('report.completion_pair', return_value=remote), \
                 patch.dict('os.environ', {'PREFLIGHT_RESULT': 'success', 'BUILD_RESULT': 'failure',
                                           'LAYOUT_RESULT': 'skipped'}):
                self.assertEqual(report(root), 1)
            outcome = json.loads((root / 'public/final-pair.json').read_text())
            self.assertEqual(outcome['remote'], remote)
            self.assertEqual(outcome['jobs'], {'preflight': 'success', 'build': 'failure', 'layout': 'skipped'})
            self.assertEqual(outcome['gates'], {'semantic_preflight': False, 'napi_parity': False})
            self.assertFalse(outcome['eligible'])


if __name__ == '__main__':
    unittest.main()
