"""One instructions:u hardware-counter attempt on this process's own child."""
import ctypes
import errno
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import time

out = Path(__file__).resolve().parent
target = out/'counter-capability.json'
assert not target.exists(), 'Preserve the original capability attempt'
assert platform.machine() == 'x86_64'
binary = out/'baseline-facts'
fixture = Path('<WORKSPACE>/no-mistakes-1972/test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts')
gate_read, gate_write = os.pipe()
stdout_read, stdout_write = os.pipe()
stderr_read, stderr_write = os.pipe()
pid = os.fork()
if pid == 0:
    os.close(gate_write)
    os.close(stdout_read)
    os.close(stderr_read)
    os.dup2(stdout_write, 1)
    os.dup2(stderr_write, 2)
    os.sched_setaffinity(0, {0})
    allowed = os.read(gate_read, 1)
    if allowed != b'1':
        os._exit(0)
    os.execve(binary, [str(binary), str(fixture), 'values', '1000'], dict(os.environ, RAYON_NUM_THREADS='1'))
os.close(gate_read)
os.close(stdout_write)
os.close(stderr_write)
attr = ctypes.create_string_buffer(128)
struct.pack_into('IIQ', attr, 0, 0, 128, 1)  # PERF_TYPE_HARDWARE, PERF_COUNT_HW_INSTRUCTIONS
struct.pack_into('Q', attr, 32, 3)  # TOTAL_TIME_ENABLED | TOTAL_TIME_RUNNING
flags = (1 << 0) | (1 << 1) | (1 << 5) | (1 << 6) | (1 << 12)
struct.pack_into('Q', attr, 40, flags)  # disabled, inherit, exclude kernel/hypervisor, enable on exec
libc = ctypes.CDLL(None, use_errno=True)
libc.syscall.restype = ctypes.c_long
fd = libc.syscall(298, ctypes.byref(attr), ctypes.c_int(pid), ctypes.c_int(-1), ctypes.c_int(-1), ctypes.c_ulong(8))
error = ctypes.get_errno() if fd < 0 else None
record = dict(event='instructions:u', syscall='perf_event_open', syscall_number=298,
    own_child=True, attr_size=128, attr_flags=flags, read_format=3,
    kernel_or_host_settings_changed=False, rayon_workers=1, child_cpu_affinity=[0],
    binary=str(binary), binary_sha256=hashlib.file_digest(binary.open('rb'), 'sha256').hexdigest(),
    fixture=str(fixture), iterations=1000, demand='values')
if fd < 0:
    record.update(available=False, errno=error, error_name=errno.errorcode.get(error), error=os.strerror(error),
                  benchmark_executed=False)
    os.write(gate_write, b'0')
    os.waitpid(pid, 0)
else:
    start = time.perf_counter()
    os.write(gate_write, b'1')
    _, status, usage = os.wait4(pid, 0)
    value, enabled, running = struct.unpack('QQQ', os.read(fd, 24))
    record.update(available=True, benchmark_executed=True, instructions=value,
        counter_time_enabled_ns=enabled, counter_time_running_ns=running,
        wall_s=time.perf_counter()-start, cpu_s=usage.ru_utime+usage.ru_stime,
        rss_kib=usage.ru_maxrss, exit=os.waitstatus_to_exitcode(status))
    os.close(fd)
os.close(gate_write)
record['stdout'] = os.read(stdout_read, 1048576).decode()
record['stderr'] = os.read(stderr_read, 1048576).decode()
os.close(stdout_read)
os.close(stderr_read)
target.write_text(json.dumps(record, indent=2)+'\n')
print(json.dumps(record, indent=2))
