"""Keep the complete consumer except the two explicitly excluded annotation rules."""
import json
from pathlib import Path

import inputs
import common

RULE = 'postgres-require-query-annotation'


def select_rules(original):
    rules = original['rules']
    removed = [row for row in rules if row.get('rule') == RULE]
    selected = [row for row in rules if row.get('rule') != RULE]
    return dict(original, rules=selected), removed


def derive_config(root, target):
    import yaml
    saved = Path(__file__).resolve().parents[3] / 'fixtures/performance/current-arm/non-annotation'
    proof = json.loads((saved / 'contract.json').read_text())
    source = Path(root) / '.no-mistakes.yml'
    assert common.digest(source) == proof['originalConfigSha256']
    original = yaml.safe_load(source.read_text())
    derived, removed = select_rules(original)
    assert len(original['rules']) == proof['originalRuleApplications'] == 133
    assert len(derived['rules']) == proof['cohortRuleApplications'] == 131
    assert [row['name'] for row in removed] == proof['excludedNames']
    assert derived == yaml.safe_load((saved / 'config.yml').read_text())
    assert {key: value for key, value in original.items() if key != 'rules'} == {
        key: value for key, value in derived.items() if key != 'rules'}
    target = Path(target)
    assert not target.exists()
    target.write_bytes((saved / 'config.yml').read_bytes())
    assert common.digest(target) == proof['cohortConfigSha256']
    # Explicit config paths select their parent's .gitignore augmentation.
    # Preserve that lookup while leaving the consumer tree unmodified.
    ignore = target.parent / '.gitignore'
    assert not ignore.exists()
    assert common.digest(Path(root) / '.gitignore') == proof['siblingGitignoreSha256']
    ignore.write_bytes((saved / 'consumer.gitignore').read_bytes())
    assert common.digest(ignore) == proof['siblingGitignoreSha256']
    return proof
