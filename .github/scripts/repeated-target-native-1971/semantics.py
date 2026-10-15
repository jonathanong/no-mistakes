"""Untimed, private semantic preflight; public payload contains hashes and counts."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

from pair import HERE, SPEC
from common import digest, require, write, failure_record


def sha(data):
    return hashlib.sha256(data).hexdigest()


def normalized(data, source):
    return json.loads(data.decode().replace(str(source), '<fixture>'))


def cli_report(directory, source, diagnostics, operation):
    code = int((directory / '0.exit').read_text())
    data = (directory / '0.stdout').read_bytes()
    report = normalized(data, source)
    if operation == 'check':
        require(set(report) == {'react', 'queues', 'rules', 'integration', 'codebase', 'warnings', 'advisories'}, 'unexpected_cli_schema')
        require(all(isinstance(value, list) for value in report.values()), 'unexpected_cli_field_shape')
        require(not report['warnings'], 'cli_warnings_present')
    else:
        require(set(report) == {'kind', 'entry', 'callSites', 'byCategory'} and
                isinstance(report['callSites'], list) and isinstance(report['byCategory'], dict), 'unexpected_effects_schema')
        require(code == 0, 'effects_cli_nonzero_exit')
    require(code in (0, 1), 'unexpected_cli_exit')
    stderr = (directory / '0.stderr').read_bytes()
    require(bool(stderr) == diagnostics, 'diagnostics_stderr_contract')
    return report, code


def preflight(root):
    private = root / 'private/semantics'
    private.mkdir(parents=True)
    overlay = root / 'inputs' / SPEC['inputs']['configured-consumer']['path'] / '.no-mistakes.yml'
    shutil.copyfile(HERE / 'consumer-overlay.yml', overlay)
    cold_overlay = root / 'inputs' / SPEC['inputs']['default-cold']['path'] / '.no-mistakes.yml'
    shutil.copyfile(HERE / 'plain-cold-overlay.yml', cold_overlay)
    signatures, cli = {}, {}
    for workload in SPEC['workloads']:
        name = workload['name']
        source = root / 'inputs' / SPEC['inputs'][workload['input']]['path']
        signatures[name], cli[name] = {}, {}
        for arm in ('baseline', 'candidate'):
            if workload['kind'] == 'probe':
                result = subprocess.run([str(root / (arm + '-timing-probe')), workload['mode'], str(source), '1'], capture_output=True)
                (private / f'{name}-{arm}.stdout').write_bytes(result.stdout)
                (private / f'{name}-{arm}.stderr').write_bytes(result.stderr)
                require(result.returncode == 0 and not result.stderr and result.stdout.strip().isdigit(), 'probe_preflight_invalid')
                signatures[name][arm] = result.stdout.decode().strip()
            else:
                directory = private / f'{name}-{arm}'
                result = subprocess.run([sys.executable, str(HERE / 'cli-worker.py'), str(root / (arm + '-cli')),
                    str(source), '1', 'on' if workload['diagnostics'] else 'off', str(directory), json.dumps(workload['command'])], capture_output=True)
                require(result.returncode == 0 and not result.stderr, 'cli_worker_preflight_invalid')
                report, code = cli_report(directory, source, workload['diagnostics'], workload['command'][0])
                signatures[name][arm] = {'report_sha256': sha(json.dumps(report, sort_keys=True).encode()), 'exit': code}
                cli[name][arm] = report
        if workload['kind'] == 'probe':
            require(signatures[name]['baseline'] == signatures[name]['candidate'], 'physical_or_fact_count_mismatch')
        elif workload.get('semantic_case') == 'plain-cold':
            require(signatures[name]['baseline'] == signatures[name]['candidate'], 'plain_cold_effect_report_mismatch')
            require(len(cli[name]['candidate']['callSites']) == 1 and cli[name]['candidate']['callSites'][0]['callee'] == 'get' and
                    cli[name]['candidate']['byCategory'] == {'read':1}, 'plain_cold_consuming_control_empty_or_changed')
        elif workload.get('semantic_case') == 'plain-hot':
            a, b = cli[name]['baseline'], cli[name]['candidate']
            require(a['callSites'] == [] and a['byCategory'] == {} and len(b['callSites']) == 1, 'plain_hot_effect_approved_delta_mismatch')
            site = b['callSites'][0]
            require(site['file'] == 'src/entry.mts' and site['line'] == 3 and site['callee'] == 'read' and
                    b['byCategory'] == {'uncategorized':1}, 'plain_hot_effect_physical_or_category_mismatch')
        elif workload['input'] == 'configured':
            require(signatures[name]['baseline'] == signatures[name]['candidate'], 'configured_cold_report_mismatch')
            require(cli[name]['candidate']['rules'], 'configured_control_has_no_findings')
        else:
            require(len(cli[name]['baseline']['rules']) == 0 and len(cli[name]['candidate']['rules']) == 1, 'consumer_approved_finding_delta_mismatch')
            finding = cli[name]['candidate']['rules'][0]
            require(finding['rule'] == 'query-reached-per-item' and finding['file'] == 'src/entry.mts' and finding['line'] == 5 and
                    'queryHelper → read' in finding['message'] and 'transaction client only' not in finding['message'], 'consumer_trace_mismatch')
            for key in set(cli[name]['candidate']) - {'rules'}:
                require(cli[name]['baseline'][key] == cli[name]['candidate'][key], 'unapproved_consumer_other_field_delta')
    for key in ('effects-plain-cold', 'effects-plain-hot'):
        require(signatures[key + '-off'] == signatures[key + '-diagnostics'], 'diagnostics_changed_report')
    # Full public graph serialization is verification-only, never native timing.
    graphs = {}
    for key in ('cold-graph', 'cold-no-calls', 'default-cold', 'hot-16', 'hot-256', 'inherited'):
        input_key = 'cold-graph' if key == 'cold-no-calls' else key
        source = root / 'inputs' / SPEC['inputs'][input_key]['path']
        graphs[key] = {}
        for arm in ('baseline', 'candidate'):
            output = private / f'{key}-{arm}-graph.json'
            with output.open('wb') as stdout, (private / f'{key}-{arm}-graph.stderr').open('wb') as stderr:
                result = subprocess.run([str(root / (arm + '-graph-probe')), str(source), '1', 'off', 'no-calls' if key == 'cold-no-calls' else 'calls'], stdout=stdout, stderr=stderr)
            require(result.returncode == 0 and not (private / f'{key}-{arm}-graph.stderr').stat().st_size, 'graph_semantic_preflight_failed')
            graphs[key][arm] = normalized(output.read_bytes(), source)['semantic']
        a, b = graphs[key]['baseline'], graphs[key]['candidate']
        require([(s['file'], s['offset'], s['caller_id'], s['invocation'], s['callee']) for s in a['sites']] ==
                [(s['file'], s['offset'], s['caller_id'], s['invocation'], s['callee']) for s in b['sites']], 'physical_graph_identity_changed')
        if key in ('cold-graph', 'cold-no-calls', 'default-cold'):
            require(a == b, 'cold_semantic_graph_changed')
            if key == 'cold-no-calls':
                require(not a['sites'] and a['nodes'] and a['edges'], 'no_calls_import_graph_empty_or_has_calls')
        elif key.startswith('hot-'):
            count = int(key.split('-')[1])
            reads = [site for site in b['sites'] if site['callee'] == 'receiver.get']
            require(len(reads) == count, 'hot_physical_count_mismatch')
            require(all(not s['unknown'] and sorted(t['name'] for t in s['known']) == ['receiver.get', 'replacement'] for s in reads), 'hot_candidate_alternatives_mismatch')
            old_reads = [site for site in a['sites'] if site['callee'] == 'receiver.get']
            require(all((s['unknown'] and not s['known']) or (not s['unknown'] and [t['name'] for t in s['known']] == ['receiver.get']) for s in old_reads), 'hot_baseline_outside_approved_boundary')
            extra = [edge for edge in b['edges'] if edge not in a['edges']]
            require(all(edge in b['edges'] for edge in a['edges']) and len(extra) <= 1 and all('replacement' in str(edge) for edge in extra), 'unapproved_hot_edge_delta')
        else:
            expected = {'localSaved':'localWriter.get', 'nestedSaved':'nestedWriter.get', 'defaultSaved':'defaultReceiver.get',
                        'savedForeign':'foreignOnly.get', 'foreignOnly.set':'foreignOnly.set', 'lexical.info':'lexical.info',
                        'nestedRepeated':None, 'lexicalEarly.get':None, 'foreignOnly.get':None, 'repeatedSaved':None, 'forwarded':None}
            for callee, target in expected.items():
                sites = [s for s in b['sites'] if s['callee'] == callee]
                require(len(sites) == 1, 'inherited_expected_site_missing')
                site = sites[0]
                require(([t['name'] for t in site['known']], site['unknown']) == ([target] if target else [], target is None), 'inherited_source_snapshot_mismatch')
    curated = {key: {arm: {'sha256':sha(json.dumps(graph, sort_keys=True).encode()), 'sites':len(graph['sites']), 'edges':len(graph['edges'])} for arm, graph in arms.items()} for key, arms in graphs.items()}
    write(root / 'public/semantic-preflight.json', {'signatures':signatures, 'graphs':curated,
        'consumer_overlay_sha256':digest(overlay), 'plain_cold_overlay_sha256':digest(cold_overlay),
        'actual_incoming':'Exact existing CI internal ParameterInputs assertions; not reconstructed by this public graph probe',
        'direct_prepared_lookup_timing':'unmeasured'})


def inspect_sample(root, directory, workload, arm, count):
    result = {'valid': False, 'stdout_sha256':digest(directory / 'stdout'), 'stdout_bytes':(directory / 'stdout').stat().st_size,
              'stderr_sha256':digest(directory / 'stderr'), 'stderr_bytes':(directory / 'stderr').stat().st_size}
    try:
        require(not result['stderr_bytes'], 'worker_stderr_present')
        expected = json.loads((root / 'public/semantic-preflight.json').read_text())['signatures'][workload['name']][arm]
        if workload['kind'] == 'probe':
            require((directory / 'stdout').read_text().strip() == expected, 'probe_signature_changed')
        else:
            require((directory / 'stdout').read_text().strip() == str(count), 'cli_batch_incomplete')
            source = root / 'inputs' / SPEC['inputs'][workload['input']]['path']
            requests = directory / 'requests'
            require(len(list(requests.glob('*.exit'))) == count, 'cli_request_count_changed')
            for index in range(count):
                # Borrow the fixed file naming contract; parsing occurs after wait4.
                report = normalized((requests / f'{index}.stdout').read_bytes(), source)
                require(sha(json.dumps(report, sort_keys=True).encode()) == expected['report_sha256'] and int((requests / f'{index}.exit').read_text()) == expected['exit'], 'cli_report_or_exit_changed')
                require(bool((requests / f'{index}.stderr').stat().st_size) == workload['diagnostics'], 'cli_diagnostics_changed')
        result['valid'] = True
    except Exception:
        result['error_code'] = 'sample_output_or_semantics_invalid'
    return result


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    root = parser.parse_args().root.resolve()
    try:
        preflight(root)
    except Exception as error:
        write(root / 'public/semantic-failure.json', failure_record(error, 'semantics'))
        sys.exit(1)
