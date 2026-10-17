"""Operational RSS derivative only; original common.run and estimator unchanged."""
from pathlib import Path
import re
from common import run,save

def child_rss(raw):
    assert re.fullmatch(rb'[0-9]+\n',raw),'GNU time receipt must be one integer KiB line'
    result=int(raw)
    assert result>0,'GNU time child RSS must be positive'
    return result

def run_with_child_rss(argv,out,seconds,env):
    out=Path(out);receipt=Path(str(out)+'.gnu-time-rss.txt')
    assert not receipt.exists(),'RSS receipt must be fresh'
    wrapped=['/usr/bin/time','--format=%M','--output='+str(receipt),'--',*argv]
    row=run(wrapped,out,seconds,env)
    # Retain the original aggregate wait4 receipt without relabelling its floor.
    save(Path(str(out)+'.wait4-original.json'),row)
    assert row['exit']==0,'failed GNU time/CLI remains failed, never a memory sample'
    row.update(wait4_wrapper_peak_rss_kib=row['rss_kib'],rss_kib=child_rss(receipt.read_bytes()),rss_source='gnu-time-child-percent-M-kib',cpu_scope='wait4-wrapper-plus-child',wall_scope='wrapper-plus-child-unchanged25ms-observer',cli_argv=argv)
    save(out.with_suffix('.json'),row)
    return row
