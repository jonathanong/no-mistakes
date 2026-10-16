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
from artifact_rules import extract_artifact, graph_probe_cfg
from measure import capture_two_arms, complete_sample_row
from final_union import curate
from napi_parity import adapter_lock_notice_count, conditional_executor_tap_counts
from common import ExperimentFailure
from postgres_source_control import tap_counts as postgres_source_tap_counts

HERE = Path(__file__).resolve().parent
OLD = HERE / 'test-fixtures' / 'original-protocol'


class ProposalTests(unittest.TestCase):
    def test_postgres_source_control_requires_all_real_passes_and_target(self):
        data = (HERE / 'test-fixtures/postgres-source-pass.tap').read_bytes()
        self.assertEqual(postgres_source_tap_counts(data), {'tests': 1, 'pass': 1, 'fail': 0, 'skipped': 0})
        for bad in (b'', data.replace(b'# pass 1', b'# pass 0'), data.replace(b'# fail 0', b'# fail 1'),
                    data.replace(b'# skipped 0', b'# skipped 1'), data.replace(b'ok 1 - compiled', b'not ok 1 - compiled'),
                    data.replace(b'legacy nested index', b'other index')):
            with self.assertRaises(ExperimentFailure):
                postgres_source_tap_counts(bad)

    def test_missing_postgres_source_control_rejects_before_git_or_build(self):
        for value in (None, {}, {'rows': []}):
            spec = json.loads((HERE / 'pair.json').read_text())
            spec['status'] = 'REVIEWED AND FROZEN'
            spec['semantic_policy']['postgres_source_control'] = value
            with patch('proposal_guard.git') as lookup, self.assertRaisesRegex(AssertionError, 'PostgreSQL source control pins'):
                check(spec, Path('/missing-source'), 'pair-preflight')
            lookup.assert_not_called()

    def test_conditional_executor_requires_one_actual_pass_and_no_skips(self):
        data = (HERE / 'test-fixtures/conditional-executor-pass.tap').read_bytes()
        self.assertEqual(conditional_executor_tap_counts(data), {'tests': 1, 'pass': 1, 'fail': 0, 'skipped': 0})
        for bad in (data.replace(b'# pass 1', b'# pass 0'),
                    data.replace(b'# skipped 0', b'# skipped 1'),
                    data.replace(b'# tests 1', b'# tests 0'),
                    data.replace(b'# fail 0', b'# fail 1'), b''):
            with self.assertRaises(ExperimentFailure):
                conditional_executor_tap_counts(bad)

    def test_missing_conditional_executor_pins_reject_before_source_or_build(self):
        for value in (None, {}, {'fixture': None}):
            spec = json.loads((HERE / 'pair.json').read_text())
            spec['status'] = 'REVIEWED AND FROZEN'
            spec['semantic_policy']['conditional_executor_control'] = value
            with patch('proposal_guard.git') as git, self.assertRaisesRegex(AssertionError, 'conditional executor control pins'):
                check(spec, Path('/missing-source'), 'pair-preflight')
            git.assert_not_called()

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
        jobs=yaml.safe_load((HERE/'workflow-proposal.yml').read_text())['jobs']
        self.assertEqual(jobs['registration']['if'],"${{ github.event_name == 'push' }}")
        self.assertEqual(jobs['eligibility']['if'],"${{ github.event_name == 'workflow_dispatch' }}")
        self.assertEqual(jobs['production-core']['needs'],'eligibility')
        self.assertEqual(jobs['layout-core']['needs'],'eligibility')
        for name,job in jobs.items():
            if name=='registration':continue
            checkout=[step for step in job['steps']if step.get('uses','').startswith('actions/checkout@')]
            self.assertEqual(len(checkout),1)
            self.assertEqual(checkout[0]['with']['fetch-depth'],0)
            self.assertFalse(checkout[0]['with']['persist-credentials'])
            self.assertEqual(job['timeout-minutes'],30)
            anchors=[i for i,step in enumerate(job['steps'])if 'entry.py bootstrap'in step.get('run','')]
            self.assertEqual(anchors,[1])

    def test_workflow_timeouts_match_reviewed_pair_budgets(self):
        spec=json.loads((HERE/'pair.json').read_text())
        self.assertEqual(spec['bounded_job']['total_seconds'],1800)
        self.assertEqual(spec['bounded_job']['sample_seconds'],1500)
        self.assertEqual(spec['bounded_job']['upload_reserve_seconds'],180)
        jobs=yaml.safe_load((HERE/'workflow-proposal.yml').read_text())['jobs']
        for name,job in jobs.items():
            for step in job['steps']:
                self.assertLessEqual(step.get('timeout-minutes',1),27)
            if name=='registration':continue
            uploads=[step for step in job['steps']if step.get('uses','').startswith('actions/upload-artifact@')]
            self.assertLessEqual(sum(step['timeout-minutes']for step in uploads),3)
        self.assertEqual(jobs['measurement']['strategy']['matrix']['cohort'],['0','1'])

    def test_baseline_graph_probe_keeps_legacy_compile_shape(self):
        self.assertIsNone(graph_probe_cfg('baseline'))
        self.assertEqual(graph_probe_cfg('candidate'), 'alternatives_api')
        source = (HERE / 'graph-probe.rs.txt').read_text()
        self.assertIn('#[cfg(not(alternatives_api))]', source)
        self.assertIn('#[cfg(alternatives_api)]', source)

    def test_graph_payload_materialization_is_exact_and_fail_closed(self):
        import hashlib
        import phase_build
        with tempfile.TemporaryDirectory()as name:
            base=Path(name);helper=base/'helper';helper.mkdir()
            payload=helper/'graph-probe.rs.txt';raw=(HERE/'graph-probe.rs.txt').read_bytes();payload.write_bytes(raw)
            pinned=hashlib.sha256(raw).hexdigest()
            (helper/'proposal-file-hashes.json').write_text(json.dumps({'files':{'graph-probe.rs.txt':pinned}}))
            with patch.object(phase_build,'HERE',helper):
                target=phase_build.materialize_graph_probe(base/'arm')
                self.assertEqual(target.read_bytes(),raw)
                self.assertEqual(json.loads((base/'arm/proof/graph-probe-materialization.json').read_text())['materialized_sha256'],pinned)
                with self.assertRaises(AssertionError):phase_build.materialize_graph_probe(base/'arm')
                payload.write_bytes(raw+b'changed')
                with self.assertRaises(AssertionError):phase_build.materialize_graph_probe(base/'other-arm')
                self.assertFalse((base/'other-arm/harness/graph-probe.rs').exists())

    def test_exact_pair_identity_and_draft_rejects_every_stage(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        self.assertEqual(spec['baseline']['revision'], 'e5ae00bd6353a28045a5344f4accac411a2ccd74')
        self.assertEqual(spec['candidate']['revision'], '23f16cd8a44268af939ea27ccf25de2b2af97a57')
        self.assertEqual(spec['candidate']['merge_parents'],
                         [spec['baseline']['revision'], spec['pr']['head_sha']])
        self.assertEqual(spec['pr']['required_successful_ci_run'], 38885017294)
        self.assertEqual(spec['pr']['head_sha'], 'bb41f4b477d9df9313f712f0a52f66a57c69a14e')
        self.assertEqual(spec['pr']['four_codecov_check_runs'], {'codecov/patch/rust': 116924075867, 'codecov/project/rust': 116924066753, 'codecov/patch/javascript': 116922492545, 'codecov/project/javascript': 116922477511})
        self.assertTrue(spec['status'].startswith('DRAFT'))
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
        spec['semantic_policy']['postgres_source_control']['source_revision'] = baseline
        for key in ('fixture', 'script'):
            spec['semantic_policy']['conditional_executor_control'][key]['source_revision'] = baseline
        spec['diagnostic_parent'] = source
        spec['semantic_policy'].update(status='REVIEWED FULL REPORTS', report_hashes={'graph': {'a': 'x'}})
        def fake_git(_repo, *args):
            if args == ('rev-parse', merge + '^{tree}'):
                return '2' * 40
            if args == ('rev-parse', source + '^{tree}'):
                return '2' * 40
            if args == ('rev-parse', baseline + '^{tree}'):
                return '1' * 40
            if args == ('show', '-s', '--format=%P', merge):
                return baseline + ' ' + source
            if args == ('show', '-s', '--format=%P', driver):
                return source
            if args[0] == 'merge-base':
                return source if args[1:]==(source,driver) else baseline
            if args[0] == 'diff':
                return 'A\t.github/scripts/repeated-target-native-2022-bounded/entry.py'
            raise AssertionError(args)
        with patch('proposal_guard.git', side_effect=fake_git)as lookup:
            with self.assertRaises(AssertionError):check(spec,Path('/unused'),'pair-preflight',driver,None)
            with self.assertRaises(AssertionError):check(spec,Path('/unused'),'pair-preflight',driver,'e'*40)
            self.assertTrue(check(spec,Path('/unused'),'pair-preflight',driver,driver))
            self.assertIn(((Path('/unused'),'merge-base',source,driver),{}),[(call.args,call.kwargs)for call in lookup.call_args_list])
            self.assertIn(((Path('/unused'),'diff','--name-status',source,driver),{}),[(call.args,call.kwargs)for call in lookup.call_args_list])
            self.assertNotIn(((Path('/unused'),'show','-s','--format=%P',driver),{}),[(call.args,call.kwargs)for call in lookup.call_args_list])
            for mode in ('outside_path','nonadditive','wrong_ancestor'):
                def invalid_git(repo,*args):
                    if args[0]=='diff':return 'A\toutside/path.py'if mode=='outside_path'else'M\t.github/scripts/repeated-target-native-2022-bounded/entry.py'if mode=='nonadditive'else fake_git(repo,*args)
                    if args==('merge-base',source,driver)and mode=='wrong_ancestor':return '0'*40
                    return fake_git(repo,*args)
                with self.subTest(mode=mode),patch('proposal_guard.git',side_effect=invalid_git),self.assertRaises(AssertionError):
                    check(spec,Path('/unused'),'pair-preflight',driver,driver)
            spec['diagnostic_parent']=merge
            with self.assertRaises(AssertionError):check(spec,Path('/unused'),'pair-preflight',driver,driver)

    def test_nonempty_pinned90_archive_preserves_all_fields(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        baseline, source, merge, driver = ('a' * 40, 'b' * 40, 'c' * 40, 'd' * 40)
        spec.update(status='REVIEWED AND FROZEN',
                    source_git_inventory_sha256={'baseline': 'e' * 64, 'candidate': 'f' * 64})
        spec['baseline'].update(revision=baseline, tree='1' * 40)
        spec['candidate'].update(revision=merge, proposed_revision=merge, tree='2' * 40, pr_head=source)
        spec['pr'].update(head_sha=source, required_successful_ci_run=1,
                          four_codecov_check_runs={name: i for i, name in enumerate(
                              ('codecov/patch/rust', 'codecov/project/rust',
                               'codecov/patch/javascript', 'codecov/project/javascript'), 1)})
        spec['semantic_policy']['postgres_source_control']['source_revision'] = baseline
        for key in ('fixture', 'script'):
            spec['semantic_policy']['conditional_executor_control'][key]['source_revision'] = baseline
        spec['diagnostic_parent'] = source
        spec['semantic_policy'].update(status='REVIEWED FULL REPORTS', report_hashes={'graph': {'a': 'x'}})
        def fake_git(_repo, *args):
            if args == ('rev-parse', merge + '^{tree}'):
                return '2' * 40
            if args == ('rev-parse', source + '^{tree}'):
                return '2' * 40
            if args == ('rev-parse', baseline + '^{tree}'):
                return '1' * 40
            if args == ('show', '-s', '--format=%P', merge):
                return baseline + ' ' + source
            if args == ('show', '-s', '--format=%P', driver):
                return source
            if args[0] == 'merge-base':
                return source if args[1:]==(source,driver) else baseline
            if args[0] == 'diff':
                return 'A\t.github/scripts/repeated-target-native-2022-bounded/entry.py'
            raise AssertionError(args)
        archive=(HERE/'test-fixtures/pinned90-input-archive.tar').read_bytes()
        with patch('proposal_guard.git',side_effect=fake_git),patch('phase_common.run',return_value=archive)as read_archive:
            self.assertTrue(check(spec,Path('/unused'),'pair-preflight',driver,driver))
            read_archive.assert_called_once()
            self.assertEqual(read_archive.call_args.args[0][:4],['git','archive','--format=tar',merge])
            self.assertEqual(len({row['path']for inp in spec['inputs'].values()for row in inp['files']}),90)
            first=next(iter(spec['inputs'].values()))['files'][0]
            for field,bad in [('lines',first['lines']+1),('bytes',first['bytes']+1),('sha256','0'*64),('path','missing/input.mts')]:
                saved=first[field];first[field]=bad
                try:
                    with self.subTest(field=field),self.assertRaises(AssertionError):
                        check(spec,Path('/unused'),'pair-preflight',driver,driver)
                finally:first[field]=saved

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

    def test_final_ledger_preserves_partial_failure_and_never_marks_ready(self):
        with tempfile.TemporaryDirectory()as name,patch.dict('os.environ',{'GITHUB_RUN_ID':'UNIT ONLY'}):
            root=Path(name)
            with self.assertRaises(Exception):curate(root)
            result=json.loads((root/'public/final-pair.json').read_text())
            self.assertFalse(result['eligible'])
            self.assertIn('INELIGIBLE',result['performance_gate'])
            self.assertIn('failure',result)


if __name__ == '__main__':
    unittest.main()
