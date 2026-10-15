import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from types import SimpleNamespace
import yaml

from proposal_guard import check
from transfer import ALLOWED, record, verify
from semantics import canonical_graph, sha
from build_arm import extract_artifact
from measure import capture_two_arms, complete_sample_row
from report import report
from napi_parity import ordered_entries
from common import ExperimentFailure


HERE = Path(__file__).resolve().parent


class ProposalTests(unittest.TestCase):
    def test_native_full_report_order_matches_collected_physical_entries(self):
        policy=json.loads((HERE/'pair.json').read_text())['semantic_policy']
        original=policy['candidate_api_effect_expectations']
        self.assertEqual(ordered_entries('typed-projections',original),list(original))
        custom=policy['custom_utility_api_effect_expectations']
        collected=['early.mts','late.mts','late-shapes.mts','shadows.mts']
        self.assertEqual(ordered_entries('imported-utility-order',custom),collected)
        self.assertNotEqual(collected,list(custom))
        for changed in ({**custom,'extra.mts':[]},
                        {key:value for key,value in custom.items() if key!='late.mts'}):
            with self.assertRaisesRegex(ExperimentFailure,'native_entry_catalog_changed'):
                ordered_entries('imported-utility-order',changed)

    def test_push_registers_without_source_or_measurement_jobs(self):
        workflow=yaml.safe_load((HERE/'workflow-proposal.yml').read_text())
        jobs=workflow['jobs']
        self.assertEqual(jobs['registration']['if'],"${{ github.event_name == 'push' }}")
        self.assertEqual(jobs['pair-preflight']['if'],"${{ github.event_name == 'workflow_dispatch' }}")
        self.assertEqual(jobs['production-build']['needs'],'pair-preflight')
        self.assertEqual(jobs['layout-build']['needs'],'pair-preflight')
        self.assertEqual(jobs['measurement']['if'],"${{ github.event_name == 'workflow_dispatch' && always() }}")
        self.assertEqual(set(jobs['measurement']['needs']),{'pair-preflight','production-build','layout-build'})

    def test_each_runtime_checkout_contains_pinned_baseline_ancestry(self):
        # The reviewed overlay is six commits past edb; a shallow depth of five
        # caused the first zero-row dispatch to fail before source verification.
        workflow=yaml.safe_load((HERE/'workflow-proposal.yml').read_text())
        for name in ('pair-preflight','production-build','layout-build','measurement'):
            with self.subTest(job=name):
                checkout=[step for step in workflow['jobs'][name]['steps']
                          if step.get('uses','').startswith('actions/checkout@')]
                self.assertEqual(len(checkout),1)
                self.assertEqual(checkout[0]['with']['fetch-depth'],0)

    def test_draft_rejects_all_stages_before_source_or_build(self):
        spec = json.loads((HERE / 'pair.json').read_text())
        spec['status']='DRAFT FAIL CLOSED: mutation control'
        for stage in ('pair-preflight', 'production-build', 'layout-build',
                      'semantic-preflight', 'napi-parity', 'sample', 'summary'):
            with self.subTest(stage=stage), self.assertRaisesRegex(AssertionError, 'draft experiment'):
                check(spec, Path('/missing-source'), stage)

    def test_reviewed_overlay_requires_external_exact_commit(self):
        spec=json.loads((HERE/'pair.json').read_text())
        spec['status']='REVIEWED AND FROZEN'
        spec['semantic_policy']['status']='REVIEWED FULL REPORTS'
        spec['semantic_policy']['report_hashes']={'graph':{}}
        spec['diagnostic_branch']='diagnostic/frozen'
        spec['pr']['required_successful_ci_run']=1
        spec['pr']['four_codecov_check_runs']={name:i for i,name in enumerate(
            ('codecov/patch/rust','codecov/project/rust',
             'codecov/patch/javascript','codecov/project/javascript'),start=1)}
        spec['inputs']={}
        head='f'*40
        def fake_git(_repo,*args):
            if args[:2]==('rev-parse',spec['candidate']['revision']+'^{tree}'):
                return spec['candidate']['tree']
            if args[:2]==('rev-parse',spec['baseline']['revision']+'^{tree}'):
                return spec['baseline']['tree']
            if args[0]=='merge-base':return spec['baseline']['revision']
            if args[0]=='show':return spec['candidate']['revision']
            if args[0]=='diff':return 'A\t.github/scripts/typed-executor-native-2060/stage.py'
            raise AssertionError(args)
        with patch('proposal_guard.git',side_effect=fake_git):
            with self.assertRaises(AssertionError):
                check(spec,Path('/unused'),'pair-preflight',head,None)
            with self.assertRaises(AssertionError):
                check(spec,Path('/unused'),'pair-preflight',head,'e'*40)
            self.assertTrue(check(spec,Path('/unused'),'pair-preflight',head,head))

    def test_private_transfer_requires_exact_bytes_and_no_extras(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            for file in ALLOWED:
                (root / file).write_bytes(file.encode())
            manifest = record(root, 'candidate', 'a' * 40, 'b' * 40, 'c' * 64,
                              'rustc 1.99.1', ['default', 'mermaid-validation'],
                              {'opt_level': 3, 'lto': True, 'codegen_units': 1})
            expected = {key: manifest[key] for key in ('arm', 'revision', 'tree',
                         'source_inventory_sha256', 'compiler', 'features', 'profile')}
            self.assertTrue(verify(root, manifest, expected))
            (root / 'cli').write_bytes(b'wrong')
            with self.assertRaises(AssertionError): verify(root, manifest, expected)
            (root / 'cli').write_bytes(b'cli')
            (root / 'source.tgz').write_bytes(b'private source')
            with self.assertRaises(AssertionError): verify(root, manifest, expected)

    def test_producer_rejects_directory_before_artifact_upload(self):
        with tempfile.TemporaryDirectory() as name:
            root=Path(name)
            for file in ALLOWED:
                (root/file).write_bytes(file.encode())
            (root/'source').mkdir()
            (root/'source'/'private.rs').write_text('secret')
            with self.assertRaises(AssertionError):
                record(root,'baseline','a'*40,'b'*40,'c'*64,'rustc 1.99.1',
                       ['default','mermaid-validation'],{'lto':True})

    def test_full_graph_hash_rejects_target_and_callable_mutations(self):
        graph = {'semantic': {'sites':[{'known':[{'name':'transaction.write','node':'["symbol","x","write","Some(CallableId(9))"]'}],
                               'callee':'query','file':'entry.mts','offset':42,'unknown':True}],
                              'nodes':['["symbol","x","write","Some(CallableId(9))"]'],
                              'edges':[], 'importer_view':[]}}
        baseline=sha(canonical_graph(json.loads(json.dumps(graph))))
        changed=json.loads(json.dumps(graph))
        changed['semantic']['sites'][0]['known'][0]['name']='other.write'
        self.assertNotEqual(sha(canonical_graph(changed)),baseline)
        changed=json.loads(json.dumps(graph))
        changed['semantic']['nodes'][0]=changed['semantic']['nodes'][0].replace('CallableId(9)','CallableId(10)')
        self.assertNotEqual(sha(canonical_graph(changed)),baseline)

    def test_owned_cli_artifact_requires_correct_target_name_and_fresh_build(self):
        source=Path('/tmp/no-mistakes/src/main.rs')
        record={'reason':'compiler-artifact','target':{'name':'no-mistakes','kind':['bin'],'src_path':str(source)},
                'fresh':False,'features':['default','mermaid-validation'],
                'profile':{'opt_level':'3','debuginfo':0,'debug_assertions':False,'overflow_checks':False,'test':False},
                'executable':'/tmp/no-mistakes/target/release/no-mistakes'}
        self.assertEqual(extract_artifact((json.dumps(record)+'\n').encode(),'bin',source),record)
        record['fresh']=True
        with self.assertRaises(Exception): extract_artifact((json.dumps(record)+'\n').encode(),'bin',source)

    def test_second_arm_startup_failure_retains_first_numeric_row(self):
        for phase in ('startup','calibration'):
            with self.subTest(phase=phase), tempfile.TemporaryDirectory() as name:
                root=Path(name)
                def fake_sample(_root,_workload,arm,_count,_affinity):
                    if arm=='candidate': raise RuntimeError('injected second-arm failure')
                    return {'arm':arm,'valid':True,'cpu_s':0.125,'wall_s':0.15,'rss_kib':100}
                with patch('measure.sample',side_effect=fake_sample):
                    with self.assertRaisesRegex(RuntimeError,'injected second-arm failure'):
                        capture_two_arms(root,{'name':'physical'},phase,1,[0,1,2,3])
                rows=json.loads((root/f'public/physical-{phase}.json').read_text())
                self.assertEqual(rows,[{'arm':'baseline','valid':True,'cpu_s':0.125,'wall_s':0.15,'rss_kib':100}])

    def test_output_inspection_failure_keeps_wait4_resources(self):
        usage=SimpleNamespace(ru_utime=0.4,ru_stime=0.1,ru_maxrss=321)
        with patch('measure.inspect_sample',side_effect=OSError('injected unreadable output')):
            row=complete_sample_row(Path('/tmp'),Path('/tmp/missing'),{'name':'physical'},
                                    'candidate',4,0,False,usage,0.7,[0,1,2,3])
        self.assertFalse(row['valid'])
        self.assertEqual(row['error_code'],'sample_output_inspection_failed')
        self.assertEqual((row['cpu_s'],row['wall_s'],row['rss_kib']),(0.5,0.7,321))

    def test_final_ledger_retains_independent_failure_reasons(self):
        with tempfile.TemporaryDirectory() as name:
            root=Path(name)
            remote={'eligible':False,'status':'historical or unverified',
                    'error_code':'candidate_remote_head_changed'}
            with patch('report.completion_pair',return_value=remote), \
                 patch.dict('os.environ',{'PREFLIGHT_RESULT':'success','BUILD_RESULT':'failure',
                                          'LAYOUT_RESULT':'skipped'}):
                self.assertEqual(report(root),1)
            outcome=json.loads((root/'public/final-pair.json').read_text())
            self.assertEqual(outcome['remote'],remote)
            self.assertEqual(outcome['jobs'],{'preflight':'success','build':'failure','layout':'skipped'})
            self.assertEqual(outcome['gates'],{'semantic_preflight':False,'napi_parity':False})
            self.assertFalse(outcome['eligible'])


if __name__ == '__main__':
    unittest.main()
