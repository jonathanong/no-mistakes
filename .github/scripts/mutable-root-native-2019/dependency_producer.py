"""Select an actual owned production dependency, never its host-build variant."""
from common import *

PROFILE = dict(opt_level='3', debuginfo=0, debug_assertions=False, overflow_checks=False, test=False)

def select_release_rlib(data, name, root, package_id):
    records = [json.loads(line) for line in data.splitlines() if line.startswith('{')]
    selected = [row for row in records if row.get('reason') == 'compiler-artifact'
                and row['target']['name'] == name
                and row['target']['kind'] == ['lib'] and row['package_id'] == package_id
                and row['profile'] == PROFILE]
    require(len(selected) == 1, 'dependency producer mismatch')
    row = selected[0]
    paths = [Path(path) for path in row['filenames'] if path.endswith('.rlib')]
    require(len(paths) == 1, 'dependency rlib cardinality')
    require(paths[0].is_absolute() and '..' not in paths[0].parts
            and paths[0].parent == root / 'target/release/deps', 'foreign dependency path')
    return row, paths[0]
