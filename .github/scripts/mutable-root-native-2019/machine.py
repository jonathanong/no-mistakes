"""Fresh hosted ARM identity; isolation is not exclusive physical hardware."""
from common import *
from budgets import cpu_quota,effective_memory,memory_limits
import platform,shutil,struct

def snapshot(build=False):
    WORK.mkdir(parents=True,exist_ok=True)
    allowed=sorted(os.sched_getaffinity(0));quota=cpu_quota();limits=memory_limits()
    memory={s.split(':')[0]:int(s.split()[1])for s in Path('/proc/meminfo').read_text().splitlines()}
    available=effective_memory(memory['MemAvailable']*1024,limits)
    require(platform.machine()=='aarch64','not ARM64')
    require(allowed[:4]==[0,1,2,3]and(quota is None or quota>=4),'four CPU entitlement unavailable')
    require(available>=(8 if build else 2)*1024**3,'insufficient effective memory')
    require(shutil.disk_usage(WORK).free>=(10 if build else 2)*1024**3,'insufficient disk')
    return {'architecture':platform.machine(),'kernel':platform.release(),'cpuinfo':Path('/proc/cpuinfo').read_text(),
            'image_os':os.environ.get('ImageOS'),'image_version':os.environ.get('ImageVersion'),
            'allowed_cpus':allowed,'selected_cpus':[0,1,2,3],'quota':quota,'memory_limits':limits,
            'effective_memory_bytes':available,'run_id':os.environ['GITHUB_RUN_ID'],'rayon_workers':4}

def elf(path):
    with Path(path).open('rb')as stream:data=stream.read(20)
    require(data[:4]==b'\x7fELF'and struct.unpack('<H',data[18:20])[0]==183,'not ARM ELF')
