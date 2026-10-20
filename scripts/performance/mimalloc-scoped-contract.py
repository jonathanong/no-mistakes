"""Pinned supplemental scopes; none can satisfy the allocator adoption matrix."""
import hashlib
import json
import subprocess
from pathlib import Path

SAVED = Path(__file__).resolve().parents[2] / 'fixtures/performance/mimalloc-scoped'
SCOPES = {'consumer-non-annotation': ('1', '4', 'max'), 'annotation-workers': ('1', 'max')}
CONSUMER = 'f38a048f5eeff0438fc59a15e9eb742def575a36'
LOCK = '000a40975d95b994e55d7c53ca43912d74dcd90f859b7010674f0661ebaf29e5'


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


def inventory(root):
    # Include installed/ignored files and symlink identities, without following links.
    rows = []
    for path in sorted(Path(root).rglob('*')):
        relative = str(path.relative_to(root))
        if '.git' in path.relative_to(root).parts:
            continue
        if path.is_symlink():
            rows.append([relative, 'link', str(path.readlink())])
        elif path.is_file():
            rows.append([relative, path.stat().st_size, digest(path)])
    return hashlib.sha256(json.dumps(rows, separators=(',', ':')).encode()).hexdigest()


def prepare(consumer, output):
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    proof = json.loads((SAVED / 'consumer/contract.json').read_text())
    consumer = Path(consumer).resolve()
    assert git(consumer, 'rev-parse', 'HEAD') == CONSUMER
    assert digest(consumer / 'pnpm-lock.yaml') == LOCK
    assert digest(consumer / '.no-mistakes.yml') == proof['originalConfigSha256']
    assert digest(consumer / '.gitignore') == proof['siblingGitignoreSha256']
    assert not git(consumer, 'status', '--porcelain', '--untracked-files=all')
    config = output / 'cohort.yml'
    config.write_bytes((SAVED / 'consumer/config.yml').read_bytes())
    (output / '.gitignore').write_bytes((SAVED / 'consumer/consumer.gitignore').read_bytes())
    assert digest(config) == proof['cohortConfigSha256']
    import yaml
    original = yaml.safe_load((consumer / '.no-mistakes.yml').read_text())
    selected = yaml.safe_load(config.read_text())
    removed = [row for row in original['rules'] if row['rule'] == proof['excludedRule']]
    assert len(original['rules']) == 133 and len(selected['rules']) == 131
    assert [row['name'] for row in removed] == proof['excludedNames']
    assert selected == dict(original, rules=[row for row in original['rules'] if row not in removed])
    annotation = SAVED / 'annotation'
    annotated = json.loads((annotation / 'contract.json').read_text())
    for name, expected in annotated['files'].items():
        assert digest(annotation / name) == expected, name
    return {
        'consumer-non-annotation': {'root': str(consumer), 'config': str(config),
            'config_sha256': digest(config), 'gitignore_sha256': digest(output / '.gitignore'),
            'workload_commit': git(consumer, 'rev-parse', 'HEAD'),
            'workload_tree': git(consumer, 'rev-parse', 'HEAD^{tree}'),
            'inventory_sha256': inventory(consumer), 'cohort': proof},
        'annotation-workers': {'root': str(annotation), 'config': str(annotation / '.no-mistakes.yml'),
            'config_sha256': digest(annotation / '.no-mistakes.yml'), 'gitignore_sha256': 'absent',
            'workload_commit': git(SAVED, 'rev-parse', 'HEAD'),
            'workload_tree': git(SAVED, 'rev-parse', 'HEAD^{tree}'),
            'inventory_sha256': inventory(annotation), 'fixture': annotated},
    }


def validate_report(row, stdout, stderr):
    assert row['exit'] == 1 and not row['timed_out'] and not row['signal']
    report = json.loads(Path(stdout).read_text())
    arrays = ('react', 'queues', 'rules', 'integration', 'codebase', 'warnings', 'advisories')
    assert all(isinstance(report.get(key), list) for key in arrays)
    assert any(report[key] for key in arrays[:5]), 'expected-finding control became empty'
    workload = row['metadata']['workload']
    text = Path(stderr).read_text()
    if workload == 'annotation-workers':
        assert len(report['rules']) == 32
        assert all(not report[key] for key in arrays if key != 'rules')
        if row['metadata']['diagnostics']:
            assert '[timing] postgres.annotation.project:' in text
            assert '[work] postgres.annotation.roots: 32' in text
    elif workload == 'consumer-non-annotation':
        assert 'postgres.annotation.project' not in text
    else:
        raise ValueError('unknown supplemental scope')
    return report
