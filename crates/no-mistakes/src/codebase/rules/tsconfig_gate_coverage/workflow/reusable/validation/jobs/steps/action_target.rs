pub(crate) fn action_target_valid(target: &str) -> bool {
    if target.contains("${{") {
        return false;
    }
    if let Some(path) = target.strip_prefix("./") {
        if target.chars().any(char::is_whitespace) {
            return false;
        }
        return !path.contains('\\')
            && (path.is_empty()
                || path
                    .split('/')
                    .all(|segment| !matches!(segment, "" | "." | "..")));
    }
    if let Some(image) = target.strip_prefix("docker://") {
        return super::super::containers::valid_container_image(image);
    }
    if target.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((path, reference)) = target.rsplit_once('@') else {
        return false;
    };
    let mut segments = path.split('/');
    segments
        .next()
        .is_some_and(super::super::super::valid_remote_owner)
        && segments
            .next()
            .is_some_and(super::super::super::valid_remote_repository)
        && segments.all(|segment| !segment.is_empty())
        && super::super::super::valid_remote_reference(reference)
}
