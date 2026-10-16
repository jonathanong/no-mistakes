"""Lossless producer metadata carry; never fabricate freshness or transfer Git credentials."""
from common import *
import tarfile,shutil

def directory(arm):
    require(arm in ARM,'unknown arm')
    return WORK/arm

def metadata(path):
    data=os.readlink(path).encode()if path.is_symlink()else None
    return {'sha256':hashlib.sha256(data).hexdigest()if data is not None else sha(path),'bytes':len(data)if data is not None else path.stat().st_size,'mode':path.lstat().st_mode&0o777,
            'kind':'symlink'if path.is_symlink()else'file','mtime':path.lstat().st_mtime}

def identity(arm,phase):
    return {'run_id':os.environ['GITHUB_RUN_ID'],'arm':arm,'revision':SPEC[arm]['revision'],
            'tree':SPEC[arm]['tree'],'toolchain':SPEC['toolchain'],'target':SPEC['target'],
            'features':SPEC['features'],'profile':SPEC['profile'],'phase':phase}

def seal(arm,phase):
    root=directory(arm);transfer=WORK/'transfer'/arm;transfer.mkdir(parents=True,exist_ok=True)
    paths=[]
    # Only tracked source bytes; .git/config, credentials, process environment never enter state.
    for name in git('ls-files',cwd=root/'source').splitlines():paths.append(root/'source'/name)
    for name in ['target','binaries','proof']:
        prefix=root/name
        if prefix.exists():paths.extend(p for p in prefix.rglob('*')if p.is_file()or p.is_symlink())
    record=identity(arm,phase);record['files']={}
    for path in sorted(paths):
        require(path.resolve().is_relative_to(root),'state symlink outside owned arm')
        record['files'][str(path.relative_to(root))]=metadata(path)
    archive=transfer/'state.tar'
    with tarfile.open(archive,'w',format=tarfile.PAX_FORMAT)as tar:
        for name in record['files']:tar.add(root/name,arcname=name,recursive=False)
    record['archive_sha256']=sha(archive);write(transfer/'state.json',record)
    return record

def restore(arm,previous):
    root=directory(arm);transfer=WORK/'incoming'/arm
    manifest=json.loads((transfer/'state.json').read_text());expected=identity(arm,previous)
    require(all(manifest[k]==v for k,v in expected.items()),'state identity/profile/run mismatch')
    require(sha(transfer/'state.tar')==manifest['archive_sha256'],'state archive changed')
    tracked=set(git('ls-files',cwd=root/'source').splitlines())
    with tarfile.open(transfer/'state.tar')as tar:
        members=tar.getmembers();require({m.name for m in members}==set(manifest['files']),'state entry set changed')
        require(len(members)==len(manifest['files']),'duplicate state entry')
        for item in members:
            path=Path(item.name);target=root/path;row=manifest['files'][item.name]
            require(not path.is_absolute()and'..'not in path.parts and path.parts[0]in
                    ['source','target','binaries','proof'],'unsafe state entry')
            require(item.isfile()or item.issym(),'unsupported state type')
            require(target.parent.resolve().is_relative_to(root),'state parent escaped owned arm')
            if path.parts[0]=='source':
                require(str(Path(*path.parts[1:]))in tracked,'untracked source/credential entry')
                require(metadata(target)['sha256']==row['sha256'],'source differs from fresh Git checkout')
            else:require(not target.exists()and not target.is_symlink(),'state overwrites output')
            if item.issym():
                link=Path(item.linkname)
                require(not link.is_absolute()and(target.parent/link).resolve().is_relative_to(root),
                        'state symlink escapes owned arm')
            target.parent.mkdir(parents=True,exist_ok=True)
            if target.is_symlink():target.unlink()
            # Standard archive extraction restores authentic producer mode/mtime/link metadata.
            # No depinfo/fingerprint edits, manufactured timestamps or forced Cargo freshness.
            tar.extract(item,path=root,filter='data')
            actual=metadata(target)
            require(all(actual[k]==row[k]for k in ['sha256','bytes','kind','mode']),'state payload mismatch')
            require(abs(actual['mtime']-item.mtime)<0.000001,'producer archive metadata lost')
    require(not git('status','--porcelain',cwd=root/'source'),'restored source differs from Git')
    return manifest

def cleanup(arm):
    target=directory(arm)/'target';require(not target.is_symlink(),'target symlink')
    if target.exists():shutil.rmtree(target)
