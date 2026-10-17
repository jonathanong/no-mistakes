"""Hash every Rust Debug token; only anonymous maps/sets ignore entry order.

Named structs, arrays, tuples, scalar spelling, and quoted bytes remain ordered.
This is an offline investigation helper, not an approved workflow gate.
"""
import hashlib
import re

IDENTIFIER = re.compile(rb"[A-Za-z_][A-Za-z_0-9]*(?:::[A-Za-z_][A-Za-z_0-9]*)*\Z")
CLOSE = {40: 41, 91: 93, 123: 125}


class Frame:
    def __init__(self, opener=None, named=False):
        self.opener = opener
        self.named = named
        self.item = hashlib.sha256(b"item-v1")
        self.tokens = 0
        self.colons = 0
        self.previous_identifier = False
        self.items = []

    def atom(self, token):
        self.item.update(b"A" + len(token).to_bytes(8, "big") + token)
        self.tokens += 1
        self.colons += token == b":"
        self.previous_identifier = bool(IDENTIFIER.fullmatch(token))

    def group(self, digest):
        self.item.update(b"G" + digest)
        self.tokens += 1
        self.previous_identifier = False

    def finish_item(self):
        if not self.tokens:
            raise ValueError("empty item before comma")
        self.items.append((self.item.digest(), self.colons))
        self.item = hashlib.sha256(b"item-v1")
        self.tokens = self.colons = 0
        self.previous_identifier = False

    def finish(self):
        if self.tokens:
            self.finish_item()
        unordered = self.opener == 123 and not self.named
        if unordered and self.items:
            colons = {n for _, n in self.items}
            if colons not in ({0}, {1}):
                raise ValueError("anonymous collection has malformed entries")
        entries = [digest for digest, _ in self.items]
        if unordered:
            entries.sort()  # Preserve duplicate entries; never build a dict/set.
        tag = repr((self.opener, self.named, unordered)).encode()
        digest = hashlib.sha256(b"group-v1" + tag + len(entries).to_bytes(8, "big"))
        for entry in entries:
            digest.update(entry)
        return digest.digest(), unordered, len(entries)


def digest_debug(data):
    frames = [Frame()]
    position = 0
    stats = dict(atoms=0, groups=0, unordered_groups=0, entries=0, max_depth=0)
    while position < len(data):
        char = data[position]
        if char in b" \t\r\n":
            position += 1
            continue
        frame = frames[-1]
        if char in CLOSE:
            frames.append(Frame(char, char == 123 and frame.previous_identifier))
            stats["max_depth"] = max(stats["max_depth"], len(frames) - 1)
            position += 1
            continue
        if char in CLOSE.values():
            if len(frames) == 1 or CLOSE[frame.opener] != char:
                raise ValueError("mismatched closing delimiter")
            value, unordered, count = frames.pop().finish()
            frames[-1].group(value)
            stats["groups"] += 1
            stats["unordered_groups"] += unordered
            stats["entries"] += count
            position += 1
            continue
        if char == 44:
            frame.finish_item()
            position += 1
            continue
        if char in (34, 39):
            start = position
            position += 1
            while position < len(data) and data[position] != char:
                position += 2 if data[position] == 92 else 1
            if position >= len(data):
                raise ValueError("unterminated quoted token")
            position += 1
            token = data[start:position]
        elif char == 58 and data[position:position + 2] != b"::":
            token = b":"
            position += 1
        else:
            start = position
            while position < len(data):
                current = data[position]
                if current in b" \t\r\n{}[](),\"'":
                    break
                if current == 58:
                    if data[position:position + 2] != b"::":
                        break
                    position += 2
                else:
                    position += 1
            if position == start:
                raise ValueError("unrecognized token")
            token = data[start:position]
        frame.atom(token)
        stats["atoms"] += 1
    if len(frames) != 1:
        raise ValueError("unclosed delimiter")
    value, _, count = frames[0].finish()
    stats["entries"] += count
    return dict(sha256=value.hex(), stats=stats)
