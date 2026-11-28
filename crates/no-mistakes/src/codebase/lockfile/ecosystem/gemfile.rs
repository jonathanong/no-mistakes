use super::super::{ResolutionKind, ResolvedPackage};
use super::gemfile_fields::{checksum_fingerprint, is_gem_version};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GemfileError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Specs(ResolutionKind),
    Checksums,
    Other,
}

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, GemfileError> {
    let mut out: Vec<ResolvedPackage> = Vec::new();
    let mut section = Section::None;
    let mut saw_header = false;
    let mut section_fingerprint = "";
    let mut source_glob = None;
    let mut git_submodules = false;
    for raw in content.lines() {
        let line = raw.trim_end_matches('\r');
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let trimmed = line.trim();
        if ["<<<<<<<", "|||||||", "=======", ">>>>>>>"]
            .iter()
            .any(|marker| trimmed.starts_with(marker))
        {
            return Err(GemfileError);
        }
        if trimmed.is_empty() {
            continue;
        }
        if indent == 0 {
            section_fingerprint = "";
            source_glob = None;
            git_submodules = false;
            if is_header(trimmed) {
                section = match trimmed {
                    "GEM" => Section::Specs(ResolutionKind::Registry),
                    "GIT" => Section::Specs(ResolutionKind::Git),
                    "PATH" => Section::Specs(ResolutionKind::Directory),
                    "PLUGIN SOURCE" => Section::Specs(ResolutionKind::Other),
                    "CHECKSUMS" => Section::Checksums,
                    _ => Section::Other,
                };
                saw_header |= matches!(
                    trimmed,
                    "GEM"
                        | "GIT"
                        | "PATH"
                        | "PLUGIN SOURCE"
                        | "PLATFORMS"
                        | "DEPENDENCIES"
                        | "BUNDLED WITH"
                        | "RUBY VERSION"
                        | "CHECKSUMS"
                );
            } else {
                section = Section::None;
            }
            continue;
        }
        if matches!(
            section,
            Section::Specs(ResolutionKind::Git | ResolutionKind::Directory)
        ) {
            if let Some(glob) = trimmed.strip_prefix("glob:") {
                source_glob = Some(glob.trim());
            }
        }
        let source_field = match section {
            Section::Specs(ResolutionKind::Git) => {
                if let Some(value) = trimmed.strip_prefix("submodules:") {
                    git_submodules = value.trim() == "true";
                }
                trimmed.strip_prefix("revision:")
            }
            Section::Specs(ResolutionKind::Directory | ResolutionKind::Other) => {
                trimmed.strip_prefix("remote:")
            }
            _ => None,
        };
        if let Some(value) = source_field {
            section_fingerprint = value.trim();
        }
        if section == Section::Checksums {
            if let Some((name, version, Some(digest))) = spec(trimmed) {
                let Some(pkg) = out
                    .iter_mut()
                    .find(|pkg| pkg.name == name && pkg.version == version)
                else {
                    continue;
                };
                pkg.fingerprint = digest;
            }
            continue;
        }
        let Section::Specs(kind) = section else {
            continue;
        };
        if indent != 4 {
            if indent < 6 && spec(trimmed).is_some() {
                return Err(GemfileError);
            }
            continue;
        }
        let Some((name, version, _)) = spec(trimmed) else {
            return Err(GemfileError);
        };
        if !is_gem_version(version) {
            return Err(GemfileError);
        }
        let mut fingerprint = super::source_fingerprint(section_fingerprint, "glob", source_glob);
        if git_submodules {
            fingerprint.push_str("#submodules=true");
        }
        out.push(ResolvedPackage {
            name: name.to_string(),
            version: version.to_string(),
            fingerprint,
            kind,
        });
    }
    if !saw_header {
        return Err(GemfileError);
    }
    Ok(out)
}

fn is_header(line: &str) -> bool {
    line.chars().all(|ch| ch.is_ascii_uppercase() || ch == ' ')
}

fn spec(trimmed: &str) -> Option<(&str, &str, Option<String>)> {
    let open = trimmed.find(" (")?;
    let name = trimmed[..open].trim();
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    let after = &trimmed[open + 2..];
    let close = after.find(')')?;
    let version = &after[..close];
    let extra = after[close + 1..].trim();
    Some((name, version, checksum_fingerprint(extra)))
}
