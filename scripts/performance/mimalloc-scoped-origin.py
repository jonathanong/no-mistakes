"""Check the immutable physical consumer's GitHub artifact identity."""
import json
import sys
from pathlib import Path


def validate(row):
    assert row['id'] == 12158768554 and row['name'] == 'matched-consumer' and not row['expired']
    assert row['digest'] == 'sha256:080223598499563b3f21bdd29e450536db72183c5ef65e3d3f221976273c6911'
    run = row['workflow_run']
    assert run['id'] == 39189171362 and run['head_sha'] == '76b3a95fa47490371eeeaa9e0eb8794fd7a9ad98'
    assert run['repository_id'] == run['head_repository_id'] == 1235521884


if __name__ == '__main__':
    validate(json.loads(Path(sys.argv[1]).read_text()))
