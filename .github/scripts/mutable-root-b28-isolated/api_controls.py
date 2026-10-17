"""Real native API and saved declaration shim controls retain distinct TAP counts."""
import json
import os
from pathlib import Path
import re
import sys
from supervise import command
from transport import sha
import subreaper

D = Path(__file__).resolve().parent


def main():
    stage, output, deadline = Path(sys.argv[1]), Path(sys.argv[2]), float(sys.argv[3])
    subreaper.enable()
    spec = json.loads((D / 'hosted-spec.json').read_text())
    source = Path('candidate-source').resolve()
    # This is a source-only checkout of the actual published parent, not a
    # relabeling of either older compiled native producer.
    for label, args in [('api-source-head', ['rev-parse', 'HEAD']),
                        ('api-source-tree', ['rev-parse', 'HEAD^{tree}']),
                        ('api-source-status', ['status', '--porcelain', '--untracked-files=no'])]:
        command(['git', '-C', str(source), *args], label, output, deadline, 10)
    assert (output / 'api-source-head.stdout').read_text().strip() == spec['candidate_head']
    assert (output / 'api-source-tree.stdout').read_text().strip() == spec['candidate_tree']
    assert not (output / 'api-source-status.stdout').read_bytes().strip()
    counts = json.loads((D / 'api-counts.json').read_text())
    runtime = output / 'native-api-runtime'
    runtime.mkdir(mode=0o700)
    receipts = []
    for arm in ['baseline', 'candidate']:
        for name, expected in counts.items():
            if arm == 'baseline' and name == 'mutable-receiver-api.test.js':
                continue  # These 67 cases assert the intended new receiver feature.
            addon = source / 'fixtures/napi/test-addon.js' if name == 'api.test.js' else stage / (arm + '-addon.node')
            env = dict(os.environ, NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(addon), RAYON_NUM_THREADS='4')
            env['XDG_RUNTIME_DIR'] = str(runtime)
            env.pop('GH_TOKEN', None)
            label = arm + '-' + name
            command(['node', '--test', '--test-reporter=tap', str(source / 'packages/no-mistakes/scripts' / name)],
                    label, output, deadline, 120, env)
            text = (output / (label + '.stdout')).read_text()
            actual = {}
            for key in ['tests', 'pass', 'fail', 'skipped']:
                matches = re.findall(r'^# ' + key + r' (\d+)$', text, re.M)
                assert len(matches) == 1, (label, key)
                actual[key] = int(matches[0])
            assert actual == {'tests': expected, 'pass': expected, 'fail': 0, 'skipped': 0}, (label, actual)
            assert not (output / (label + '.stderr')).read_bytes()
            receipts.append({'arm': arm, 'script': name, 'counts': actual, 'addon_sha256': sha(addon),
                             'scope': 'saved declaration shim' if name == 'api.test.js' else 'real release addon'})
    (output / 'api-controls.json').write_text(json.dumps({'complete': True, 'records': receipts,
        'baseline_real_count': sum(v for k, v in counts.items() if k not in ['api.test.js', 'mutable-receiver-api.test.js']),
        'candidate_real_count': sum(v for k, v in counts.items() if k != 'api.test.js'),
        'shim_count_each_arm': counts['api.test.js'],
        'baseline_feature_expectations_excluded': 'Only 67 new mutable-receiver assertions; complete per-arm raw goldens remain mandatory'}, indent=2))


if __name__ == '__main__':
    main()
