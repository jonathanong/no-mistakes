"""Closed same-run ARM artifacts; transfer no targets or credentials to measurement."""
from common import *
from state import directory,identity
from machine import elf
import shutil

def package(arm):
    root=directory(arm);dest=WORK/'closed'/arm
    require(not dest.exists(),'closed artifact exists');dest.mkdir(parents=True)
    for phase in ['core','cli','addon','probe','sql']:
        row=json.loads((root/'proof'/(phase+'-closed.json')).read_text())
        require(row['arm']==arm and row['revision']==SPEC[arm]['revision'],'phase arm/source mismatch')
    shutil.copytree(root/'proof',dest/'proof');shutil.copytree(root/'binaries',dest/'binaries')
    shutil.copy2(HERE/'payload'/('source-'+arm+'.json'),dest/'source-manifest.json')
    producer=json.loads((dest/'proof/library-producer.json').read_text())
    require(producer['fresh']is False and producer['features']==SPEC['features'],'missing actual producer')
    for name in ['cli','addon.node','probe','sql-probe']:elf(dest/'binaries'/name)
    manifest=identity(arm,'sql');manifest.update(
        binary_sha256=sha(dest/'binaries/probe'),library_sha256=sha(dest/'binaries/lib.rlib'),
        source_manifest_sha256=sha(dest/'source-manifest.json'),compiler_artifact=producer,
        rustc=json.loads((dest/'proof/sql-closed.json').read_text())['rustc'],release_profile=SPEC['profile'],
        lock_sha256=SPEC['lock_sha256'],harness=[{'path':p.name.removesuffix('.txt'),'sha256':sha(p)}for p in sorted((HERE/'payload/original/harness').glob('*.rs.txt'))])
    manifest['files']={str(p.relative_to(dest)):sha(p)for p in sorted(dest.rglob('*'))if p.is_file()}
    manifest['file_modes']={name:(dest/name).stat().st_mode&0o777 for name in manifest['files']}
    write(dest/'manifest.json',manifest)

def verify(arm):
    dest=WORK/'closed'/arm;row=json.loads((dest/'manifest.json').read_text())
    require(all(row[k]==v for k,v in identity(arm,'sql').items()),'closed identity mismatch')
    actual={str(p.relative_to(dest))for p in dest.rglob('*')if p.is_file()and p!=dest/'manifest.json'}
    require(actual==set(row['files']),'closed artifact file set mismatch')
    restore_file_modes(dest,row['files'],row['file_modes'])
    require(row['compiler_artifact']['fresh']is False,'no actual library producer')
    require(sha(dest/'source-manifest.json')==SPEC[arm]['source_manifest_sha256']==row['source_manifest_sha256'],'source inventory mismatch')
    require(sha(dest/'binaries/probe')==row['binary_sha256']and sha(dest/'binaries/lib.rlib')==row['library_sha256'],'analyzer identities mismatch')
    require(row['lock_sha256']==SPEC['lock_sha256'],'dependency lock changed')
    require(row['compiler_artifact']['profile']==dict(opt_level='3',debuginfo=0,debug_assertions=False,overflow_checks=False,test=False),'Cargo profile changed')
    require(row['features']==SPEC['features']and row['release_profile']==SPEC['profile'],'profile mismatch')
    for name in ['cli','addon.node','probe','sql-probe']:elf(dest/'binaries'/name)
    return row


def restore_file_modes(dest,files,modes):
    # ZIP transport strips executable bits; authenticate every file before restoring any mode.
    require(set(files)==set(modes),'closed mode inventory mismatch')
    for name,digest in files.items():
        relative=Path(name);path=dest/relative
        require(not relative.is_absolute()and '..'not in relative.parts and path.resolve().is_relative_to(dest),
                'closed path escapes owned artifact')
        require(path.is_file()and not path.is_symlink()and sha(path)==digest,'closed artifact changed')
        require(type(modes[name])is int and 0<=modes[name]<=0o777,'unsafe closed file mode')
    for name,mode in modes.items():os.chmod(dest/name,mode,follow_symlinks=False)
