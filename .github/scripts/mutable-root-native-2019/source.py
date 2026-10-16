"""Fresh exact source checkout and actual Cargo artifact provenance."""
from common import *

def checkout(arm):
    source=WORK/arm/'source';revision=SPEC[arm]['revision']
    require(not source.exists(),'source already exists')
    run(['git','clone','--shared','--no-checkout',str(ROOT),str(source)],timeout=120)
    run(['git','checkout','--detach',revision],cwd=source,timeout=120)
    require(git('rev-parse','HEAD',cwd=source)==revision and
            git('rev-parse','HEAD^{tree}',cwd=source)==SPEC[arm]['tree'],'source identity changed')
    expected=json.loads((HERE/'payload'/('source-'+arm+'.json')).read_text())
    import io,tarfile
    archive=run(['git','archive',revision,'Cargo.toml','Cargo.lock','crates/no-mistakes'],cwd=source,timeout=90)
    with tarfile.open(fileobj=io.BytesIO(archive))as tree:
        members={item.name:item for item in tree if item.isfile()}
        selected={name for name in members if name in ['Cargo.toml','Cargo.lock','crates/no-mistakes/Cargo.toml']or(name.startswith('crates/no-mistakes/')and name.endswith('.rs'))}
        require(len(expected)==len({r['path']for r in expected})and {r['path']for r in expected}==selected,'incomplete source inventory')
        require(sha(HERE/'payload'/('source-'+arm+'.json'))==SPEC[arm]['source_manifest_sha256'],'source inventory changed')
        for row in expected:
            p=source/row['path'];require(p.is_file()and sha(p)==row['sha256'],'live source differs')
            require(hashlib.sha256(tree.extractfile(members[row['path']]).read()).hexdigest()==row['sha256'],'Git/live differ')
    require(sha(source/'Cargo.lock')==SPEC['lock_sha256'],'lock changed')
    return source

def select(data,name,kind):
    records=[json.loads(line)for line in data.splitlines()if line.startswith(b'{')]
    matches=[r for r in records if r.get('reason')=='compiler-artifact'and
             r['target']['name']==name and kind in r['target']['kind']]
    require(len(matches)==1,'artifact cardinality')
    row=matches[0]
    require(row['fresh']is False,'owned producer was not fresh')
    require(row['features']==SPEC['features'],'feature mismatch')
    require(row['profile']=={'opt_level':'3','debuginfo':0,'debug_assertions':False,
                            'overflow_checks':False,'test':False},'profile mismatch')
    return row,records

def compiler():
    text=run(['rustc','-Vv'],timeout=30).decode().strip()
    require(text.startswith('rustc 1.99.1 (58cf8f9c3 ')and'host: '+SPEC['target']in text,'compiler/target mismatch')
    require(run(['rustc','+stable','-Vv'],timeout=30).decode().strip()==text,'latest stable differs')
    return text
