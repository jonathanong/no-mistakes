"""Read owned target size; tolerate only entries Cargo has removed meanwhile."""
import os
import stat


def target_bytes(root):
    total = 0
    pending = [os.fspath(root)]
    while pending:
        directory = pending.pop()
        try:
            with os.scandir(directory) as entries:
                for entry in entries:
                    try:
                        info = entry.stat(follow_symlinks=False)
                    except FileNotFoundError:
                        continue
                    if stat.S_ISDIR(info.st_mode):
                        pending.append(entry.path)
                    elif stat.S_ISREG(info.st_mode):
                        total += info.st_size
                    # Symlinks and other nonregular entries contribute no bytes.
        except FileNotFoundError:
            # A discovered directory may disappear before scanning or iteration.
            continue
    return total
