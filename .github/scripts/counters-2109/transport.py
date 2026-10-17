"""Bounded allowlisted evidence directory manifests; no Git/env/secret transport."""
import hashlib,json
from pathlib import Path
MANIFEST='artifact-manifest.json'
def files(root):
    root=Path(root).resolve();result={};total=0
    for p in sorted(root.rglob('*')):
        assert not p.is_symlink(),'artifact symlink rejected'
        if not p.is_file():continue
        name=str(p.relative_to(root));assert not any(x=='.git'or x.startswith('.env')for x in p.relative_to(root).parts)
        if name==MANIFEST:continue
        total+=p.stat().st_size;assert total<=536870912 and len(result)<10000
        result[name]={'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
    return result

def seal(root):
    root=Path(root);assert root.is_dir()and not(root/MANIFEST).exists()
    (root/MANIFEST).write_text(json.dumps(files(root),indent=2)+'\n')

def verify(root):
    root=Path(root);expected=json.loads((root/MANIFEST).read_text());assert expected and files(root)==expected,'complete artifact file inventory/hash drift'
    return expected
