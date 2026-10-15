"""Optional own-child PMU capability check; never changes host settings."""
import ctypes
import errno
import os
from pathlib import Path
import platform
import struct


def metadata():
    devices = Path('/sys/bus/event_source/devices')
    return {'devices': sorted(p.name for p in devices.iterdir()),
            'paranoid': Path('/proc/sys/kernel/perf_event_paranoid').read_text().strip()}


def counter_smoke():
    number = {'x86_64': 298, 'aarch64': 241}.get(platform.machine())
    if number is None:
        return {'available': False, 'reason': 'unsupported syscall architecture'}
    read, write = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(write)
        try:
            if os.read(read, 1) == b'1':
                os.execl('/bin/true', 'true')
            os._exit(0)
        except BaseException:
            os._exit(127)
    os.close(read)
    fd = -1
    try:
        attr = ctypes.create_string_buffer(128)
        struct.pack_into('IIQ', attr, 0, 0, 128, 1)
        struct.pack_into('Q', attr, 40, (1 << 0) | (1 << 1) | (1 << 5) | (1 << 6) | (1 << 12))
        libc = ctypes.CDLL(None, use_errno=True)
        libc.syscall.restype = ctypes.c_long
        fd = libc.syscall(number, ctypes.byref(attr), ctypes.c_int(pid), ctypes.c_int(-1),
                         ctypes.c_int(-1), ctypes.c_ulong(8))
        error = ctypes.get_errno() if fd < 0 else 0
        os.write(write, b'1' if fd >= 0 else b'0')
        os.close(write)
        write = None
        _, status = os.waitpid(pid, 0)
        pid = None
        if fd < 0:
            return {'available': False, 'errno': error, 'error_name': errno.errorcode.get(error),
                    'reason': os.strerror(error)}
        if os.waitstatus_to_exitcode(status):
            return {'available': False, 'reason': 'PMU smoke child failed'}
        return {'available': True, 'smoke_native_instructions': struct.unpack('Q', os.read(fd, 8))[0]}
    finally:
        if write is not None:
            os.close(write)  # EOF also releases a gated child after a pre-exec failure.
        if pid is not None:
            os.waitpid(pid, 0)
        if fd >= 0:
            os.close(fd)


def capability():
    result = {'event': 'instructions:u', 'host_settings_changed': False}
    try:
        result.update(metadata())
    except (OSError, ValueError) as error:
        result.update(metadata_unavailable=True, metadata_error_type=type(error).__name__,
                      metadata_errno=getattr(error, 'errno', None))
    try:
        result.update(counter_smoke())
    except (OSError, ValueError, struct.error) as error:
        result.update(available=False, reason='optional PMU capability unavailable',
                      error_type=type(error).__name__, errno=getattr(error, 'errno', None))
    return result
