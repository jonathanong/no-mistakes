use crate::codebase::lockfile::ResolutionKind;

pub(super) fn resolution_info(value: &serde_yaml::Value) -> (String, ResolutionKind) {
    let Some(resolution) = value.get("resolution") else {
        return (String::new(), ResolutionKind::Other);
    };
    if resolution.get("repo").is_some() {
        return (
            resolution
                .get("commit")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            ResolutionKind::Git,
        );
    }
    if let Some(dir) = resolution.get("directory").and_then(|v| v.as_str()) {
        return (dir.to_string(), ResolutionKind::Directory);
    }
    if let Some(tarball) = resolution.get("tarball").and_then(|v| v.as_str()) {
        let fingerprint = resolution
            .get("integrity")
            .and_then(|v| v.as_str())
            .unwrap_or(tarball);
        return (fingerprint.to_string(), ResolutionKind::Tarball);
    }
    if let Some(commit) = resolution.get("commit").and_then(|v| v.as_str()) {
        return (commit.to_string(), ResolutionKind::Git);
    }
    if let Some(integrity) = resolution.get("integrity").and_then(|v| v.as_str()) {
        return (integrity.to_string(), ResolutionKind::Registry);
    }
    (String::new(), ResolutionKind::Other)
}
