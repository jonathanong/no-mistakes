"""Always retain exact allowed evidence inside two portable outer filenames."""
import json
import os
from pathlib import Path
import stat
import shutil

from cohort_transport import pack


def stage_tree(source, destination):
    if not source.exists():
        return
    assert source.is_dir() and not source.is_symlink()
    for path in sorted(source.rglob('*')):
        mode = path.lstat().st_mode
        assert stat.S_ISDIR(mode) or stat.S_ISREG(mode), 'Evidence links/special files rejected'
        if stat.S_ISREG(mode):
            target = destination / path.relative_to(source)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)  # Works across /tmp and RUNNER_TEMP filesystems.


def main():
    cohort = os.environ['COHORT']
    assert cohort in ['0', '1', '2', '3', '4', '5', '6', '7']
    temporary = Path(os.environ['RUNNER_TEMP'])
    staging = temporary / '2022-evidence-staging'
    staging.mkdir(exist_ok=False)
    controller = temporary / 'repeated-target-2022-isolated'
    stage_tree(controller, staging / 'controller')
    semantic = Path('/tmp/nm2022-main281-current-pair-functional/current-pair-semantics')
    for name in ['public', 'private']:
        stage_tree(semantic / name, staging / 'semantic' / name)
    for name in ['semantic-manifest.json', 'reviewed-expected-parity.json']:
        path = semantic / name
        if path.exists():
            assert stat.S_ISREG(path.lstat().st_mode)
            target = staging / 'semantic' / name
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
    (staging / 'retention-scope.json').write_text(json.dumps({'scope': 'Exact controller + public/private semantic/sample outputs only; no executable transfer/Git/token/environment capture', 'controller_existed': controller.exists(), 'semantic_existed': semantic.exists(), 'no_numerical_claim': True}, indent=2))
    pack(staging, temporary / '2022-evidence-carrier', cohort)


if __name__ == '__main__':
    main()
