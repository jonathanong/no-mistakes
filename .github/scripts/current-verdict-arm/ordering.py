"""Strict ordering-only config, report parity, and BCCB contracts."""
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'matched-arm'))

from inputs import (ARTIFACT_ID, ARTIFACT_ZIP_SHA256, FIXTURE_TAR_SHA256, CAP_SECONDS, CONFIG_SHA256,
                    CONSUMER, EXPECTED_FINDINGS, JOBS, DIAGNOSTICS, LEDGER,
                    MEASURE_MINUTES, ORIGINAL_CONFIG_SHA256)
from report import (aggregate_ledger, compare_feature_delta, digest, read_report,
                    validate_ledger, without_ledger)


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def derive_config(root, target, saved_config):
    import yaml
    from common import digest as file_digest
    source = Path(root) / '.no-mistakes.yml'
    assert file_digest(source) == ORIGINAL_CONFIG_SHA256
    original = yaml.safe_load(source.read_text())
    saved = yaml.safe_load(Path(saved_config).read_text())
    rules = original.get('rules')
    assert isinstance(rules, list) and len(rules) == 133
    selected = [row for row in rules if row.get('rule') == 'postgres-conflict-ordering']
    assert len(selected) == 1
    assert selected[0]['name'] == 'production PostgreSQL writers acquire conflicts in catalog order'
    derived = dict(original)
    derived['rules'] = selected
    assert derived == saved
    assert set(derived) == set(original)
    assert {key: value for key, value in derived.items() if key != 'rules'} == {
        key: value for key, value in original.items() if key != 'rules'}
    target = Path(target)
    target.parent.mkdir(parents=True, exist_ok=True)
    assert not target.exists()
    target.write_bytes(Path(saved_config).read_bytes())
    assert sha256(target) == CONFIG_SHA256
    return {'consumer': CONSUMER, 'originalConfigSha256': ORIGINAL_CONFIG_SHA256,
            'orderingConfigSha256': CONFIG_SHA256, 'originalRuleApplications': len(rules),
            'selectedRuleApplications': len(selected), 'selectedRule': selected[0]['name'],
            'unchangedNonRulePayload': True, 'configPathOutsideFixture': True}


def report_parity(base_path, candidate_path, ledger_on):
    base, candidate = read_report(base_path), read_report(candidate_path)
    if ledger_on:
        if 'sqlVerdicts' in base:
            raise ValueError('baseline unexpectedly contains sqlVerdicts')
        ledger = validate_ledger(candidate)
        normalized = without_ledger(candidate)
    else:
        if 'sqlVerdicts' in base or 'sqlVerdicts' in candidate:
            raise ValueError('ledger field appeared with ledger flag off')
        ledger = None
        normalized = candidate
    delta = compare_feature_delta(base, normalized)
    added = {(row['file'], row['line'], row['rule'], row.get('target'))
             for row in delta['added_policy_findings']}
    if added != EXPECTED_FINDINGS:
        raise ValueError('ordering-only feature delta differs from the two saved findings')
    if ledger is None:
        return {'featureDelta': delta, 'ledgerValidated': False}
    return {'featureDelta': delta, 'ledgerValidated': True,
            'ledgerSha256': digest(json.dumps(ledger, sort_keys=True,
                                              separators=(',', ':'), ensure_ascii=False).encode()),
            'ledger': aggregate_ledger(ledger)}


def sample_plan(jobs, diagnostics, ledger):
    assert jobs in JOBS and diagnostics in DIAGNOSTICS and ledger in LEDGER
    enabled = ledger == 'on'
    return [('base', False, 'b'), ('candidate', enabled, 'c'),
            ('candidate', enabled, 'c'), ('base', False, 'b')]


def validate_repeats(rows, output, ledger_on):
    if len(rows) != 4 or not all(row.get('eligible') for row in rows):
        raise ValueError('BCCB block is incomplete or censored')
    if [row['arm'] for row in rows] != ['b', 'c', 'c', 'b']:
        raise ValueError('BCCB arm order changed')
    if len({row['exit'] for row in rows}) != 1:
        raise ValueError('exit status changed within BCCB block')
    if len({row['semanticStderr'] for row in rows}) != 1:
        raise ValueError('semantic stderr changed within BCCB block')
    if rows[0]['stdoutSha256'] != rows[3]['stdoutSha256']:
        raise ValueError('baseline repeat report hashes differ')
    if rows[1]['stdoutSha256'] != rows[2]['stdoutSha256']:
        raise ValueError('candidate repeat report hashes differ')
    first = report_parity(Path(output) / rows[0]['stdout'],
                          Path(output) / rows[1]['stdout'], ledger_on)
    reverse = report_parity(Path(output) / rows[3]['stdout'],
                            Path(output) / rows[2]['stdout'], ledger_on)
    return {'firstPair': first, 'reversePair': reverse,
            'rawRepeatHashesMatch': True, 'semanticStderrMatch': True,
            'exitStatusMatch': True}


def run_bccb(plan, sample_arm, output, ledger_on, rows, save_rows):
    """Sample sequentially and reject a bad first B/C pair before repeats."""
    for ordinal, (role, use_ledger, arm) in enumerate(plan):
        row = sample_arm(role, use_ledger, arm, ordinal)
        row['sourceRole'] = role
        row['ledgerRequested'] = use_ledger
        rows.append(row)
        save_rows(rows)
        if ordinal == 1:
            if not all(item.get('eligible') for item in rows):
                raise ValueError('initial B/C pair is incomplete or censored')
            report_parity(Path(output) / rows[0]['stdout'],
                          Path(output) / rows[1]['stdout'], ledger_on)
    return rows


def validate_fixture_contract(provenance, archive_sha256):
    import json
    proof = json.loads(Path(provenance).read_text())
    assert proof['consumer'] == CONSUMER
    assert proof['originalConfigSha256'] == ORIGINAL_CONFIG_SHA256
    assert proof['orderingConfigSha256'] == CONFIG_SHA256
    assert proof['selectedRuleNames'] == [
        'production PostgreSQL writers acquire conflicts in catalog order']
    assert proof['originalRuleApplications'] == 133
    assert proof['selectedRuleApplications'] == 1
    assert proof['unchangedNonRulePayload'] is True
    assert ARTIFACT_ID == 12158768554
    assert proof['artifactZipSha256'] == ARTIFACT_ZIP_SHA256
    assert proof['fixtureTarSha256'] == FIXTURE_TAR_SHA256
    assert archive_sha256 == FIXTURE_TAR_SHA256
