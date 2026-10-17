"""Authenticate fixed official original ZIPs against download-action materialization."""
import hashlib,json,subprocess,sys,tempfile,zipfile
from pathlib import Path,PurePosixPath
P=Path(__file__).resolve().parent
REPO='jonathanong/no-mistakes'
def digest(b):return hashlib.sha256(b).hexdigest()
def inventory_archive(z):
    result={};total=0
    for entry in z.infolist():
        name=PurePosixPath(entry.filename)
        assert not name.is_absolute() and '..' not in name.parts and '\\' not in entry.filename
        assert (entry.external_attr>>16)&0o170000 != 0o120000
        if entry.is_dir():continue
        assert entry.filename not in result and len(result)<1000
        total+=entry.file_size;assert total<=512*1024*1024
        result[entry.filename]=(entry.file_size,digest(z.read(entry)))
    return result

def main():
    root=Path(sys.argv[1]);pair=sys.argv[2];assert pair in ['2127','2129']
    pins=json.loads((P/'frozen-native.json').read_text());assert pins['run']==39039054266
    proofs=[]
    for arm in ['baseline','candidate']:
        name=f'api-native-{pair}-{arm}';pin=next(a for a in pins['artifacts']if a['name']==name)
        endpoint=f'repos/{REPO}/actions/artifacts/{pin["id"]}'
        actual=json.loads(subprocess.check_output(['gh','api',endpoint],timeout=30))
        for k in ['id','name','size_in_bytes','digest']:assert actual[k]==pin[k]
        assert not actual['expired'] and actual['workflow_run']['id']==pins['run'] and actual['workflow_run']['head_sha']==pins['source']
        assert 0<pin['size_in_bytes']<64*1024*1024
        with tempfile.TemporaryFile() as stream:
            subprocess.run(['gh','api',endpoint+'/zip'],stdout=stream,check=True,timeout=120)
            size=stream.tell();assert size==pin['size_in_bytes'];stream.seek(0);h=hashlib.file_digest(stream,'sha256').hexdigest();assert 'sha256:'+h==pin['digest'];stream.seek(0)
            with zipfile.ZipFile(stream)as z:expected=inventory_archive(z)
        directory=root/f'{pair}-{arm}';files={str(x.relative_to(directory)):(x.stat().st_size,digest(x.read_bytes()))for x in directory.rglob('*')if x.is_file()}
        assert not any(x.is_symlink()for x in directory.rglob('*')) and files==expected
        proofs.append({'id':pin['id'],'name':name,'zip_sha256':h,'zip_bytes':size,'files':expected})
    (root/'frozen-authentication.json').write_text(json.dumps({'original_run':pins['run'],'source':pins['source'],'proofs':proofs},indent=2)+'\n')
if __name__=='__main__':main()
