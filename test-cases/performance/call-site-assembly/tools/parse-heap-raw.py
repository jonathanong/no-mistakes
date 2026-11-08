#!/usr/bin/env python3
"""Stream a heaptrack raw zstd trace and summarize live bytes at peak RSS."""

from __future__ import annotations

import argparse
import bisect
import json
import os
import subprocess
import sys
from array import array
from collections import defaultdict


class TraceError(Exception):
    pass


def parse_int(token: str, base: int, line_no: int, label: str) -> int:
    try:
        return int(token, base)
    except ValueError as exc:
        raise TraceError(f"line {line_no}: invalid {label}: {token!r}") from exc


def parse_module(tokens: list[str], line_no: int) -> dict | None:
    if len(tokens) >= 3 and tokens[2] == "-":
        return None
    if len(tokens) < 4:
        raise TraceError(f"line {line_no}: short module record")
    path_length = parse_int(tokens[1], 16, line_no, "module path length")
    if tokens[2] == "x":
        # The main executable mapping refers to its path through x/X metadata.
        path = "x"
        base_index = 3
    else:
        path = tokens[2]
        base_index = 3
    if path_length != len(path.encode("utf-8")):
        raise TraceError(f"line {line_no}: module path byte length does not match")
    if len(tokens) <= base_index:
        raise TraceError(f"line {line_no}: missing module path/base")
    base = parse_int(tokens[base_index], 16, line_no, "module base")
    rest = tokens[base_index + 1 :]
    if len(rest) % 2:
        raise TraceError(f"line {line_no}: odd module segment list")
    ranges = []
    for offset_token, size_token in zip(rest[::2], rest[1::2]):
        offset = parse_int(offset_token, 16, line_no, "segment offset")
        size = parse_int(size_token, 16, line_no, "segment size")
        if size:
            ranges.append((base + offset, base + offset + size))
    return {"path": path, "base": base, "ranges": ranges}


def read_trace(path: str) -> dict:
    proc = subprocess.Popen(["zstd", "-dc", "--", path], stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, text=True, errors="strict")
    assert proc.stdout is not None
    version = None
    page_size = None
    frame_ips = array("Q", [0])
    frame_parents = array("Q", [0])
    modules: list[dict] = []
    live: dict[int, tuple[int, int]] = {}
    live_by_trace: dict[int, int] = defaultdict(int)
    live_bytes = 0
    heap_peak_live_bytes = 0
    heap_peak_clock = None
    allocations = frees = unknown_frees = 0
    clock_records = rss_records = 0
    final_clock = final_rss = None
    peak_rss = peak_clock = None
    peak_trace_bytes: dict[int, int] | None = None
    peak_live_bytes = None
    line_no = 0
    last_tags: list[str] = []
    try:
        for raw in proc.stdout:
            line_no += 1
            tokens = raw.split()
            if not tokens:
                continue
            tag = tokens[0]
            last_tags = (last_tags + [tag])[-2:]
            if tag == "v":
                if version is not None or len(tokens) < 3:
                    raise TraceError(f"line {line_no}: duplicate or malformed version")
                version = tokens[1:]
            elif tag == "m":
                module = parse_module(tokens, line_no)
                if module is not None:
                    modules.append(module)
            elif tag == "t":
                if len(tokens) != 3:
                    raise TraceError(f"line {line_no}: malformed trace-frame record")
                frame_id = len(frame_ips)
                ip = parse_int(tokens[1], 16, line_no, "instruction pointer")
                parent = parse_int(tokens[2], 16, line_no, "parent frame id")
                if parent < 0 or parent >= frame_id:
                    raise TraceError(f"line {line_no}: invalid parent frame id {parent}")
                frame_ips.append(ip)
                frame_parents.append(parent)
            elif tag == "+":
                if len(tokens) != 4:
                    raise TraceError(f"line {line_no}: malformed allocation record")
                size = parse_int(tokens[1], 16, line_no, "allocation size")
                trace_id = parse_int(tokens[2], 16, line_no, "allocation trace id")
                ptr = parse_int(tokens[3], 16, line_no, "allocation pointer")
                if trace_id <= 0 or trace_id >= len(frame_ips):
                    raise TraceError(f"line {line_no}: unknown allocation trace id {trace_id}")
                if ptr in live:
                    raise TraceError(f"line {line_no}: pointer allocated twice without free: {ptr:x}")
                live[ptr] = (size << 64) | trace_id
                live_by_trace[trace_id] += size
                live_bytes += size
                allocations += 1
                if live_bytes > heap_peak_live_bytes:
                    heap_peak_live_bytes = live_bytes
                    heap_peak_clock = final_clock
            elif tag == "-":
                if len(tokens) != 2:
                    raise TraceError(f"line {line_no}: malformed free record")
                ptr = parse_int(tokens[1], 16, line_no, "freed pointer")
                prior = live.pop(ptr, None)
                if prior is None:
                    unknown_frees += 1
                    continue
                size, trace_id = prior >> 64, prior & ((1 << 64) - 1)
                live_by_trace[trace_id] -= size
                if live_by_trace[trace_id] == 0:
                    del live_by_trace[trace_id]
                live_bytes -= size
                frees += 1
            elif tag == "c":
                if len(tokens) != 2:
                    raise TraceError(f"line {line_no}: malformed clock record")
                final_clock = parse_int(tokens[1], 16, line_no, "clock")
                clock_records += 1
            elif tag == "R":
                if len(tokens) != 2:
                    raise TraceError(f"line {line_no}: malformed RSS record")
                pages = parse_int(tokens[1], 16, line_no, "RSS pages")
                rss = pages * (page_size or 4096)
                if rss < 0:
                    raise TraceError(f"line {line_no}: negative RSS")
                final_rss = rss
                rss_records += 1
                if peak_rss is None or rss > peak_rss:
                    peak_rss = rss
                    peak_clock = final_clock
                    peak_live_bytes = live_bytes
                    peak_trace_bytes = dict(live_by_trace)
            elif tag in ("x", "X", "I"):
                # Executable/cwd/build-id metadata does not affect live accounting.
                if tag == "I" and len(tokens) >= 2:
                    page_size = parse_int(tokens[1], 16, line_no, "page size")
                continue
            else:
                raise TraceError(f"line {line_no}: unsupported record tag {tag!r}")
    except Exception:
        proc.stdout.close()
        proc.kill()
        proc.wait()
        raise
    proc.stdout.close()
    stderr = proc.stderr.read() if proc.stderr else ""
    exit_code = proc.wait()
    if exit_code:
        raise TraceError(f"zstd failed with exit {exit_code}: {stderr.strip()}")
    if version is None:
        raise TraceError("missing version record; trace is incomplete")
    if clock_records == 0 or rss_records == 0 or final_clock is None or final_rss is None:
        raise TraceError("missing final clock/RSS records; trace is incomplete")
    if last_tags != ["c", "R"]:
        raise TraceError(f"trace does not end with final clock/RSS records: {last_tags}")
    if peak_trace_bytes is None or peak_live_bytes is None:
        raise TraceError("no RSS records; trace is incomplete")
    accounted = sum(live_by_trace.values())
    if accounted != live_bytes or sum(peak_trace_bytes.values()) != peak_live_bytes:
        raise TraceError("live byte accounting invariant failed")
    return {
        "version": version,
        "line_count": line_no,
        "frame_count": len(frame_ips) - 1,
        "module_count": len(modules),
        "allocation_count": allocations,
        "matched_free_count": frees,
        "unknown_free_count": unknown_frees,
        "final_clock": final_clock,
        "final_rss_bytes": final_rss,
        "clock_record_count": clock_records,
        "rss_record_count": rss_records,
        "peak_rss": peak_rss,
        "peak_clock": peak_clock,
        "heap_peak_live_bytes": heap_peak_live_bytes,
        "heap_peak_clock": heap_peak_clock,
        "live_bytes_at_rss_peak": peak_live_bytes,
        "live_pointer_count_at_eof": len(live),
        "live_bytes_at_eof": live_bytes,
        "peak_trace_bytes": peak_trace_bytes,
        "frame_ips": frame_ips,
        "frame_parents": frame_parents,
        "modules": modules,
    }


def load_symbols(binary: str) -> tuple[list[int], list[str]]:
    proc = subprocess.run(["nm", "-n", "-C", binary], check=True, text=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    addresses, names = [], []
    for line in proc.stdout.splitlines():
        fields = line.split(maxsplit=2)
        if len(fields) == 3 and fields[1] in "TtWwVv":
            try:
                addresses.append(int(fields[0], 16))
                names.append(fields[2])
            except ValueError:
                continue
    return addresses, names


def summarize(result: dict, binary: str | None, top_n: int) -> dict:
    symbols = ([], []) if binary is None else load_symbols(binary)
    symbol_addresses, symbol_names = symbols
    target_realpath = None if binary is None else os.path.realpath(binary)
    modules = result.pop("modules")
    frame_ips = result.pop("frame_ips")
    frame_parents = result.pop("frame_parents")
    mapped_target = []
    if target_realpath:
        for module in modules:
            if module["path"] == "x" or os.path.realpath(module["path"]) == target_realpath:
                mapped_target.append(module)

    def frame_name(ip: int) -> str:
        if not mapped_target or not symbol_addresses:
            return f"0x{ip:x}"
        module = next((item for item in mapped_target
                       if any(start <= ip < end for start, end in item["ranges"])), None)
        if module is None:
            return f"0x{ip:x}"
        offset = ip - module["base"]
        index = bisect.bisect_right(symbol_addresses, offset) - 1
        return f"{symbol_names[index]}+0x{offset - symbol_addresses[index]:x}" if index >= 0 else f"0x{ip:x}"

    stack_rows = []
    lineages = defaultdict(int)
    frame_bytes: dict[str, int] = defaultdict(int)
    groups: dict[str, int] = defaultdict(int)
    selectors = {
        "mock_site_index": ("build_mock_site_index",),
        "callsite_collection": ("call_site", "callsite", "initial_call"),
        "callsite_refinement": ("refine_value", "refinement", "replacement"),
    }
    trace_bytes = result.pop("peak_trace_bytes")
    for trace_id, size in trace_bytes.items():
        current = trace_id
        frames = []
        seen_names = set()
        seen_groups = set()
        visited = 0
        while current:
            if current <= 0 or current >= len(frame_ips):
                raise TraceError(f"peak stack references missing frame {current}")
            name = frame_name(frame_ips[current])
            if name not in seen_names:
                frame_bytes[name] += size
                seen_names.add(name)
            frames.append(name)
            lowered = name.lower()
            for group, needles in selectors.items():
                if group not in seen_groups and any(needle in lowered for needle in needles):
                    groups[group] += size
                    seen_groups.add(group)
            current = frame_parents[current]
            visited += 1
            if visited >= len(frame_ips):
                raise TraceError("cycle in trace-frame parent chain")
        stack_rows.append({"trace": trace_id, "bytes": size, "frames": frames})
        category = "other"
        for name in frames:
            if "no_mistakes::" not in name:
                continue
            if "dependencies::graph::" in name:
                category = "graph"; break
            if "dependencies::extract::" in name:
                category = "extraction"; break
            if "check_facts::" in name:
                category = "check-facts"; break
            if "source_store::" in name or "read_utf8_arc" in name:
                category = "sources"; break
        lineages[category] += size
    result["top_stacks"] = sorted(stack_rows, key=lambda row: -row["bytes"])[:top_n]
    result["exclusive_lineages"] = dict(lineages)
    result["symbol_binary"] = binary
    result["target_module_count"] = len(mapped_target)
    result["matched_inclusive_bytes_by_group"] = dict(sorted(groups.items()))
    result["top_inclusive_frames"] = [
        {"frame": name, "bytes": size}
        for name, size in sorted(frame_bytes.items(), key=lambda item: (-item[1], item[0]))[:top_n]
    ]
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace", help="heaptrack raw .zst file")
    parser.add_argument("--binary", help="release ELF for nm symbolization")
    parser.add_argument("--top", type=int, default=100)
    args = parser.parse_args()
    try:
        result = summarize(read_trace(args.trace), args.binary, args.top)
    except (OSError, subprocess.SubprocessError, TraceError) as exc:
        print(f"heap raw parse failed: {exc}", file=sys.stderr)
        return 2
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
