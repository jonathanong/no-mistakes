"""One finite offline fan-in. Missing/failed blocks remain failed, never resampled."""
import json,subprocess,sys
from pathlib import Path
from common import package_guard,save
import os
P=Path(__file__).resolve().parent
package_guard(P,os.environ)
root=Path(sys.argv[1]);out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
paths=[root/('hotcounter-block-'+case+'-'+str(block))/'block'for case in ['consumer-off','consumer-on','control-off','control-on']for block in range(4)]
save(out/'expected-blocks.json',[str(p)for p in paths])
missing=[str(p)for p in paths if not(p/'terminal.json').is_file()]
if missing:
    save(out/'terminal.json',{'status':'FAILED_INCOMPLETE','missing':missing});raise SystemExit(1)
try:
    subprocess.run([sys.executable,str(P/'assess.py'),*[x for p in paths for x in ['--block',str(p)]],'--output',str(out/'assessment.json')],check=True,timeout=60)
    save(out/'terminal.json',{'status':'COMPLETE_OFFLINE_EVIDENCE','automatic_acceptance':False})
except BaseException as error:
    save(out/'terminal.json',{'status':'FAILED','error':repr(error)});raise
