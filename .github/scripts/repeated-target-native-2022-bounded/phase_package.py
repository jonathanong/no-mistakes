"""Closed same-run artifacts retain actual producers, not fabricated freshness."""
from phase_common import *
from phase_state import directory
from transfer import record
import shutil


def package(arm, rustc):
    root=directory(arm);dest=root/'transfer';dest.mkdir(exist_ok=False)
    for phase in ('core','cli','addon','timing','graph'):
        row=json.loads((root/'proof'/(phase+'-closed.json')).read_text())
        require(row['arm']==arm and row['revision']==SPEC[arm]['revision']
                and row['tree']==SPEC[arm]['tree'], 'phase_source_identity_changed')
    for name in ('cli','addon.node','timing-probe','graph-probe'):
        source=root/'binaries'/name
        data=run(['readelf','-h',str(source)],timeout=30).decode()
        require('AArch64' in data,'non_ARM_closed_binary')
        shutil.copy2(source,dest/name)
    producers=[json.loads((root/'proof'/name).read_text()) for name in
        ('library-producer.json','cli-producer.json','addon-producer.json')]
    require(all(row['fresh'] is False for row in producers),'missing_true_Cargo_producer')
    (dest/'cargo.jsonl').write_text(''.join(json.dumps(row)+'\n' for row in producers))
    rows=json.loads((HERE/'payload'/('source-'+arm+'.json')).read_text())
    write(dest/'source-inventory.json',{'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],
        'rows':rows,'sha256':SPEC['source_git_inventory_sha256'][arm]})
    manifest=record(dest,arm,SPEC[arm]['revision'],SPEC[arm]['tree'],SPEC['source_git_inventory_sha256'][arm],
        rustc,SPEC['features'],SPEC['release_profile'],'alternatives_api' if arm=='candidate' else None)
    write(WORK/'public'/('production-'+arm+'-closed.json'),{'run_id':os.environ['GITHUB_RUN_ID'],
        'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],'manifest_sha256':sha(dest/'transfer-manifest.json'),
        'actual_producers':producers,'compiler':rustc})
    return manifest
