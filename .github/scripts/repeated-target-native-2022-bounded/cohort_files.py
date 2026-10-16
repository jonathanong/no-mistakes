"""Self-describing, same-run cohort evidence; authenticate before union accounting."""
from phase_common import *


def close(root,cohort):
    public=root/'public';files={}
    for path in sorted(public.rglob('*')):
        require(not path.is_symlink(),'cohort_public_symlink')
        if path.is_file() and path!=public/'cohort-artifact-manifest.json':
            files[str(path.relative_to(public))]={'sha256':sha(path),'bytes':path.stat().st_size}
    write(public/'cohort-artifact-manifest.json',{'run_id':os.environ['GITHUB_RUN_ID'],
        'cohort':cohort,'diagnostic_commit':os.environ['APPROVED_DIAGNOSTIC_COMMIT'],'files':files})


def verify(public,cohort):
    manifest=json.loads((public/'cohort-artifact-manifest.json').read_text())
    require(manifest['cohort']==cohort and manifest['run_id']==os.environ['GITHUB_RUN_ID']
        and manifest['diagnostic_commit']==os.environ['APPROVED_DIAGNOSTIC_COMMIT'],'foreign_cohort_artifact')
    require(not any(path.is_symlink()for path in public.rglob('*')),'cohort_public_symlink')
    actual={str(path.relative_to(public))for path in public.rglob('*')if path.is_file()
        and path!=public/'cohort-artifact-manifest.json'}
    require(actual==set(manifest['files']),'cohort_public_inventory_changed')
    for name,row in manifest['files'].items():
        path=public/name
        require(not Path(name).is_absolute() and '..'not in Path(name).parts
            and not path.is_symlink() and path.resolve().is_relative_to(public),'unsafe_cohort_public_path')
        require(sha(path)==row['sha256'] and path.stat().st_size==row['bytes'],'cohort_public_bytes_changed')
    return manifest
