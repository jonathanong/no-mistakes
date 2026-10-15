"""Bounded memtrack 1.6 stream validation; never treats decode failure as EOF."""
import hashlib
import resource
import subprocess
import tempfile
from collections import Counter
from pathlib import Path

import msgpack

COMPRESSED_LIMIT = 4 * 1024**2
DECODED_LIMIT = 16 * 1024**2
EVENT_LIMIT = 100_000
STACK_LIMIT = 4096
STACK_BYTES_LIMIT = 8 * 1024**2
KINDS = {
    'Malloc', 'Free', 'Realloc', 'Calloc', 'AlignedAlloc', 'Fork', 'Exec', 'Exit',
    'Rss', 'Rmap', 'Mmap', 'Munmap', 'Brk', 'Mapping', 'Stack',
}


class IncompleteCapture(ValueError):
    pass


def require(value, reason):
    if not value:
        raise IncompleteCapture(reason)


def pairs(items):
    result = {}
    for key, value in items:
        require(isinstance(key, str) and key not in result, 'duplicate or non-string map key')
        result[key] = value
    return result


def extension(_code, _data):
    raise IncompleteCapture('unsupported MessagePack extension')


def uint(value):
    return type(value) is int and 0 <= value < 2**64


def validate(event):
    require(type(event) is dict and event.get('type') in KINDS, 'unknown event kind')
    require(all(uint(event.get(key)) for key in ('pid', 'tid', 'timestamp', 'addr')),
            'invalid common event fields')
    require(0 < event['pid'] < 2**31 and 0 < event['tid'] < 2**31, 'invalid process identity')
    kind = event['type']
    common = {'pid', 'tid', 'timestamp', 'addr', 'type'}
    extras = {
        'Malloc': {'size', 'stack_hash'}, 'Calloc': {'size', 'stack_hash'},
        'AlignedAlloc': {'size', 'stack_hash'}, 'Realloc': {'old_addr', 'size', 'stack_hash'},
        'Free': set(), 'Exec': set(), 'Exit': set(), 'Fork': {'parent_pid'},
        'Rss': {'member', 'size'}, 'Rmap': {'member', 'delta'},
        'Mmap': {'size'}, 'Munmap': {'size'}, 'Brk': {'size'},
        'Mapping': {'path', 'dev', 'ino', 'file_offset', 'len'},
        'Stack': {'hash', 'sp', 'regs', 'bytes', 'fp_chain', 'truncated'},
    }[kind]
    require(set(event) <= common | extras, 'unsupported event field')
    if kind in ('Malloc', 'Calloc', 'AlignedAlloc', 'Realloc', 'Rss', 'Mmap', 'Munmap', 'Brk'):
        require(uint(event.get('size')), 'invalid allocation size')
    if 'stack_hash' in event:
        require(uint(event['stack_hash']), 'invalid stack hash')
    if kind == 'Realloc' and event.get('old_addr') is not None:
        require(uint(event['old_addr']), 'invalid old allocation address')
    if kind == 'Fork':
        require(type(event.get('parent_pid')) is int and 0 < event['parent_pid'] < 2**31,
                'invalid parent process')
    if kind in ('Rss', 'Rmap'):
        require(type(event.get('member')) is int and -(2**31) <= event['member'] < 2**31,
                'invalid memory member')
    if kind == 'Rmap':
        require(type(event.get('delta')) is int and -(2**63) <= event['delta'] < 2**63,
                'invalid resident memory delta')
    if kind == 'Mapping':
        require(type(event.get('path')) is str and len(event['path'].encode()) <= 4096,
                'invalid mapping path')
        require(all(uint(event.get(key)) for key in ('dev', 'ino', 'file_offset', 'len')),
                'invalid mapping coordinates')
    if kind == 'Stack':
        require(uint(event.get('hash')) and uint(event.get('sp')), 'invalid stack identity')
        require(type(event.get('bytes')) is bytes and len(event['bytes']) <= 8192,
                'invalid captured stack bytes')
        require(type(event.get('truncated')) is bool, 'invalid stack truncation flag')
        for key in ('regs', 'fp_chain'):
            require(type(event.get(key)) is list and len(event[key]) <= 256
                    and all(uint(value) for value in event[key]), 'invalid stack vector')
    return event


def unpack_chunks(chunks, *, expected_events=None):
    unpacker = msgpack.Unpacker(
        raw=False, max_buffer_size=65536, max_array_len=256, max_map_len=32,
        max_str_len=4096, max_bin_len=8192, max_ext_len=0,
        object_pairs_hook=pairs, ext_hook=extension,
    )
    events = []
    total = complete_end = stack_bytes = stacks = 0
    pids, tids = set(), set()
    for chunk in chunks:
        total += len(chunk)
        require(total <= DECODED_LIMIT, 'decompressed byte limit')
        unpacker.feed(chunk)
        for event in unpacker:
            validate(event)
            require(len(events) < EVENT_LIMIT, 'event count limit')
            pids.add(event['pid'])
            tids.add((event['pid'], event['tid']))
            require(len(pids) <= 64 and len(tids) <= 256, 'process/thread count limit')
            if event['type'] == 'Stack':
                stacks += 1
                stack_bytes += len(event['bytes'])
                require(stacks <= STACK_LIMIT and stack_bytes <= STACK_BYTES_LIMIT,
                        'stack count/byte limit')
                # Preserve validated payload identity without retaining raw stack memory.
                event = dict(event, bytes_sha256=hashlib.sha256(event['bytes']).hexdigest(),
                             bytes_length=len(event['bytes']))
                del event['bytes']
            events.append(event)
            complete_end = unpacker.tell()
    require(complete_end == total, 'truncated trailing MessagePack object')
    require(events, 'empty capture')
    if expected_events is not None:
        require(len(events) == expected_events, 'capture producer count mismatch')
    return events, {
        'events': len(events), 'decompressed_bytes': total,
        'kinds': dict(Counter(event['type'] for event in events)),
        'stack_bytes': stack_bytes,
        'timestamp_reversals': sum(a['timestamp'] > b['timestamp']
                                  for a, b in zip(events, events[1:])),
        'ordering': 'encoded input order; never assumed globally timestamp sorted',
        'producer_count_verified': expected_events is not None,
        'capture_completeness': 'producer count verified' if expected_events is not None
                                else 'unknown: no producer event count',
    }


def decoder_limits():
    resource.setrlimit(resource.RLIMIT_FSIZE, (DECODED_LIMIT, DECODED_LIMIT))
    resource.setrlimit(resource.RLIMIT_AS, (128 * 1024**2, 128 * 1024**2))
    resource.setrlimit(resource.RLIMIT_CPU, (5, 5))


def decode(path, *, expected_events=None):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), 'capture must be a regular file')
    require(path.stat().st_size <= COMPRESSED_LIMIT, 'compressed byte limit')
    with path.open('rb') as source:
        data = source.read(COMPRESSED_LIMIT + 1)
    require(len(data) <= COMPRESSED_LIMIT, 'compressed byte limit')
    # zstd decodes the concatenated stream; frame boundaries are never object boundaries.
    with tempfile.TemporaryFile() as output:
        result = subprocess.run(
            ['zstd', '--memory=8MB', '--decompress', '--stdout'],
            input=data, stdout=output, stderr=subprocess.PIPE, preexec_fn=decoder_limits,
            timeout=10, check=False,
        )
        require(result.returncode == 0, 'incomplete or invalid zstd stream')
        output.seek(0)
        events, report = unpack_chunks(iter(lambda: output.read(16384), b''),
                                       expected_events=expected_events)
    report['compressed_bytes'] = len(data)
    report['compressed_sha256'] = hashlib.sha256(data).hexdigest()
    return events, report
