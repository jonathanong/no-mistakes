"""Parity and conservative SQL proof-ledger accounting for #2215."""

from collections import Counter, defaultdict
import hashlib
import json


LEDGER_FIELDS = ('profile', 'rule', 'application', 'file', 'span', 'subcheck')
STATUSES = {'violation', 'proven-clear', 'abstain'}
EVIDENCE = {'exact', 'exhaustive-union', 'verified-prefix', 'policy-exemption'}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_report(path):
    with open(path, 'rb') as source:
        report = json.load(source)
    if not isinstance(report, dict):
        raise ValueError('check report is not a JSON object')
    for field in ('advisories', 'codebase', 'integration', 'queues', 'react', 'rules', 'warnings'):
        if not isinstance(report.get(field), list):
            raise ValueError(f'check report lacks list field {field}')
    return report


def without_ledger(report):
    if 'sqlVerdicts' not in report:
        raise ValueError('flagged report lacks sqlVerdicts')
    copy = dict(report)
    del copy['sqlVerdicts']
    return copy


def validate_ledger(report):
    rows = report.get('sqlVerdicts')
    if not isinstance(rows, list):
        raise ValueError('sqlVerdicts is not an array')
    identities = set()
    for row in rows:
        if not isinstance(row, dict) or not all(field in row for field in LEDGER_FIELDS):
            raise ValueError('sqlVerdicts row lacks identity fields')
        if row.get('status') not in STATUSES:
            raise ValueError(f"unexpected SQL proof status: {row.get('status')}")
        if row.get('evidence') is not None and row['evidence'] not in EVIDENCE:
            raise ValueError(f"unexpected SQL proof evidence: {row['evidence']}")
        if (not isinstance(row['span'], list) or len(row['span']) != 2 or
                not all(isinstance(value, int) for value in row['span'])):
            raise ValueError('SQL proof span must contain two byte offsets')
        if not isinstance(row.get('residualReasons', []), list) or not all(
                isinstance(reason, str) for reason in row.get('residualReasons', [])):
            raise ValueError('SQL residual reasons must be strings')
        identity = tuple(canonical(row[field]) for field in LEDGER_FIELDS)
        if identity in identities:
            raise ValueError(f'duplicate SQL proof obligation: {identity!r}')
        identities.add(identity)
    return rows


def aggregate_ledger(rows):
    groups = defaultdict(lambda: {'statuses': Counter(), 'evidence': Counter(),
                                  'residual_reasons': Counter(), 'rows': 0})
    sites = set()
    reasons_by_rule_subcheck = defaultdict(Counter)
    for row in rows:
        identity = tuple(canonical(row[field]).decode() for field in LEDGER_FIELDS)
        group = groups[identity]
        group['rows'] += 1
        group['statuses'][row['status']] += 1
        group['evidence'][row.get('evidence') or 'none'] += 1
        for reason in row.get('residualReasons', []):
            group['residual_reasons'][reason] += 1
            reasons_by_rule_subcheck[(row['rule'], row['subcheck'])][reason] += 1
        sites.add((identity[0], identity[3], identity[4]))
    details = []
    for identity, group in sorted(groups.items()):
        details.append({**{field: json.loads(value) for field, value in zip(LEDGER_FIELDS, identity)},
                        'rows': group['rows'], 'statuses': dict(sorted(group['statuses'].items())),
                        'evidence': dict(sorted(group['evidence'].items())),
                        'residual_reasons': dict(sorted(group['residual_reasons'].items()))})
    return {'ledger_rows': len(rows), 'represented_executor_sites': len(sites),
            'application_count': len({(item['rule'], canonical(item['application'])) for item in details}),
            'status_counts': dict(sorted(Counter(row['status'] for row in rows).items())),
            'evidence_counts': dict(sorted(Counter(row.get('evidence') or 'none' for row in rows).items())),
            'residual_reasons_by_rule_subcheck': [
                {'rule': rule, 'subcheck': subcheck, 'counts': dict(sorted(reasons.items()))}
                for (rule, subcheck), reasons in sorted(reasons_by_rule_subcheck.items())],
            'by_rule_application_profile_file_span_subcheck': details,
            'standalone_sql_fragments': 'unrepresented by this executor-site ledger',
            'suspicious_config_candidates': 'unmeasured by this executor-site ledger',
            'newly_applied_calls': 'unmeasured without a matched baseline call/finding map'}


def compare_reports(base_path, other_path, *, with_ledger):
    base = read_report(base_path)
    other = read_report(other_path)
    if with_ledger:
        if 'sqlVerdicts' in base:
            raise ValueError('flag-off report unexpectedly contains sqlVerdicts')
        ledger = validate_ledger(other)
        normalized = without_ledger(other)
        if canonical(base) != canonical(normalized):
            raise ValueError('flag changed pre-existing findings, warnings, advisories, or fields')
        return {'normalized_sha256': digest(canonical(normalized)),
                'ledger_sha256': digest(canonical(ledger)), 'ledger': aggregate_ledger(ledger)}
    if 'sqlVerdicts' in base or 'sqlVerdicts' in other:
        raise ValueError('flag-off report unexpectedly contains sqlVerdicts')
    with open(base_path, 'rb') as first, open(other_path, 'rb') as second:
        if first.read() != second.read():
            raise ValueError('baseline/candidate default reports differ byte-for-byte')
    return {'normalized_sha256': digest(canonical(base)), 'ledger_sha256': None}
