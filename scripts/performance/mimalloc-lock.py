"""Keep canonical lock identity and actual checkout bytes distinct in release proof."""
import hashlib


def lock_identity(actual, tracked):
    # Git may check out Cargo.lock with CRLF on Windows. Accept only that
    # conversion; a modified dependency graph must never borrow HEAD's identity.
    if actual.replace(b'\r\n', b'\n') != tracked:
        raise ValueError('working copy differs from tracked Cargo.lock')
    return {'lockSha256': hashlib.sha256(tracked).hexdigest(),
            'lockFileSha256': hashlib.sha256(actual).hexdigest()}
