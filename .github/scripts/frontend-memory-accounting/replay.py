"""Observed allocator-object accounting; unknown histories never become retention."""
from collections import Counter

from decode import require

ALLOCATIONS = {'Malloc', 'Calloc', 'AlignedAlloc'}
LIVE_LIMIT = 32768


def replay(events, pid, begin, end, *, addresses=None, continuous_capture=False, producer_verified=False, history_verified=False):
    require(type(pid) is int and pid > 0, 'invalid selected process')
    require(0 <= begin < end <= len(events), 'invalid marker window')
    # Restrict initialization to the last observed image, never join reused process images.
    image = max((i for i, event in enumerate(events[:begin])
                 if event['pid'] == pid and event['type'] == 'Exec'), default=-1)
    live = {}
    live_bytes = 0
    generations = 0
    problems = Counter()
    allocated = freed = alloc_calls = free_calls = unknown_window_frees = 0
    before = peak = None
    for index in range(max(image, 0), end):
        if index == begin:
            before = live_bytes
            peak = before
        event = events[index]
        if event['pid'] != pid:
            continue
        kind, address = event['type'], event['addr']
        inside = begin <= index < end
        if kind in ('Exec', 'Fork', 'Exit'):
            if inside:
                problems['process image/lifecycle changed inside window'] += 1
            if kind == 'Exec':
                live.clear()
                live_bytes = 0
            continue
        if kind not in ALLOCATIONS | {'Free', 'Realloc'}:
            continue
        old = event.get('old_addr') if kind == 'Realloc' else None
        if addresses is not None and address not in addresses and old not in addresses:
            continue
        # A null free is a no-op. Null allocation results are not emitted by memtrack1.6.
        if kind == 'Free' and not address:
            continue
        if kind == 'Realloc' and not address:
            problems['null realloc result: old object disposition unknown'] += 1
            if inside:
                unknown_window_frees += 1
            continue
        if kind == 'Realloc' and old is None:
            problems['realloc missing old address'] += 1
            if inside:
                unknown_window_frees += 1
        released = old if kind == 'Realloc' else address if kind == 'Free' else None
        if released:
            prior = live.pop(released, None)
            if prior is None:
                problems['unmatched free/realloc object'] += 1
                if inside:
                    unknown_window_frees += 1
            if prior is not None:
                live_bytes -= prior[1]
            if inside:
                free_calls += 1
                if prior is not None:
                    freed += prior[1]
        if kind in ALLOCATIONS | {'Realloc'}:
            if not address:
                problems['null allocation result'] += 1
                continue
            if address in live:
                problems['address reused before observed release'] += 1
                live_bytes -= live[address][1]
            generations += 1
            live[address] = (generations, event['size'])
            live_bytes += event['size']
            require(len(live) <= LIVE_LIMIT, 'live object count limit')
            if inside:
                allocated += event['size']
                alloc_calls += 1
        if inside:
            peak = max(peak, live_bytes)
    require(before is not None, 'missing marker start')
    complete = (producer_verified and continuous_capture and history_verified
                and image >= 0 and not problems)
    return {
        'scope': 'explicit ledger object subset' if addresses is not None
                 else 'observed allocator events for one process image',
        'producer_count_verified': producer_verified,
        'external_object_history_verified': history_verified,
        'arithmetic_scope': 'observed selected events only; net is never retained-object identity',
        'allocation_convention': 'successful realloc counts old-size release and new-size allocation',
        'window_allocated_bytes': allocated,
        'window_matched_freed_bytes': freed,
        'window_freed_bytes': None if unknown_window_frees else freed,
        'window_allocation_calls': alloc_calls,
        'window_release_calls': free_calls,
        'window_net_bytes': None if unknown_window_frees else allocated - freed,
        'known_live_at_begin_bytes': before,
        'known_live_at_end_bytes': live_bytes,
        'observed_stream_peak_bytes': peak if complete else None,
        'observed_live_sum_max_bytes': peak,
        'retained_objects': len(live) if complete else None,
        'retention_complete': complete,
        'history_problems': dict(problems),
        'continuity': 'externally attested continuous capture' if continuous_capture
                      else 'unknown: raw events do not prove tracking was continuously enabled',
        'peak_scope': 'encoded stream order only; not CodSpeed server peak or native RSS',
    }
