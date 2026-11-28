const PRE: &[&[u8]] = &[b"alpha", b"beta", b"preview", b"rc", b"a", b"b", b"c"];
const POST: &[&[u8]] = &[b"post", b"rev", b"r"];
const DEV: &[&[u8]] = &[b"dev"];

pub(super) fn pep440(version: &str) -> bool {
    let (public, local) = match version.split_once('+') {
        Some((public, local)) => (public, Some(local)),
        None => (version, None),
    };
    if local.is_some_and(|local| !local_label(local)) {
        return false;
    }
    let bytes = public.as_bytes();
    let mut index = 0;
    if let Some(bang) = public.find('!') {
        if bang == 0 || !bytes[..bang].iter().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        index = bang + 1;
    }
    if !release(bytes, &mut index) {
        return false;
    }
    take_tag(bytes, &mut index, PRE);
    take_tag(bytes, &mut index, POST);
    take_tag(bytes, &mut index, DEV);
    index == bytes.len()
}

fn local_label(local: &str) -> bool {
    !local.is_empty()
        && local
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_alphanumeric()))
}

fn release(bytes: &[u8], index: &mut usize) -> bool {
    if !digits(bytes, index) {
        return false;
    }
    while bytes.get(*index) == Some(&b'.') {
        let dot = *index;
        *index += 1;
        if !digits(bytes, index) {
            *index = dot;
            break;
        }
    }
    true
}

fn take_tag(bytes: &[u8], index: &mut usize, labels: &[&[u8]]) -> bool {
    let start = *index;
    if matches!(bytes.get(*index), Some(b'.' | b'_' | b'-')) {
        *index += 1;
    }
    let Some(label) = labels.iter().find(|label| {
        bytes
            .get(*index..*index + label.len())
            .is_some_and(|slice| slice.eq_ignore_ascii_case(label))
    }) else {
        *index = start;
        return false;
    };
    *index += label.len();
    if matches!(bytes.get(*index), Some(b'.' | b'_' | b'-')) {
        *index += 1;
    }
    if digits(bytes, index) {
        return true;
    }
    *index = start;
    false
}

fn digits(bytes: &[u8], index: &mut usize) -> bool {
    let start = *index;
    while bytes.get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    *index > start
}
