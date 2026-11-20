use super::options::{CacheSpec, Options};
use super::output::proven_output_names;
use super::scan::{Field, Job, Step};
use super::text::{has_fragment, hash_args, norm_path, path_entries};
use super::RuleFinding;
use super::RULE_ID;

pub(super) fn judge(rel: &str, jobs: &[Job], options: &Options) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for job in jobs {
        for (index, step) in job.steps.iter().enumerate() {
            if !step
                .uses
                .as_deref()
                .is_some_and(super::text::is_cache_action)
            {
                continue;
            }
            let paths = step
                .path
                .as_ref()
                .map(|field| path_entries(&field.value))
                .unwrap_or_default();
            for cache in &options.caches {
                let path = norm_path(&cache.path);
                if !paths.iter().any(|entry| entry == &path) {
                    continue;
                }
                judge_step(
                    rel,
                    step,
                    &job.steps[..index],
                    cache,
                    &path,
                    options,
                    &mut findings,
                );
            }
        }
    }
    findings
}

fn judge_step(
    rel: &str,
    step: &Step,
    earlier: &[Step],
    cache: &CacheSpec,
    path: &str,
    options: &Options,
    findings: &mut Vec<RuleFinding>,
) {
    match step.key.as_ref() {
        None => findings.push(hit(rel, step.line, missing_key(path))),
        Some(key) if key.value.trim().is_empty() => {
            findings.push(hit(rel, key.line, missing_key(path)))
        }
        Some(key) => {
            let missing: Vec<_> = cache
                .key_parts
                .iter()
                .map(|part| part.trim())
                .filter(|part| !has_fragment(&key.value, part))
                .map(str::to_string)
                .collect();
            if !missing.is_empty() {
                findings.push(hit(rel, key.line, platform_message(path, &missing)));
            }
            if !digest_proven(
                &key.value,
                &cache.lockfile,
                earlier,
                options.allow_hash_files,
            ) {
                findings.push(hit(
                    rel,
                    key.line,
                    digest_message(path, &norm_path(&cache.lockfile), &key.value, options),
                ));
            }
        }
    }
    if options.reject_restore_keys && restore_present(step.restore_keys.as_ref()) {
        let line = step
            .restore_keys
            .as_ref()
            .map(|field| field.line)
            .unwrap_or(step.line);
        findings.push(hit(rel, line, restore_message(path)));
    }
}

fn digest_proven(key: &str, lockfile: &str, earlier: &[Step], allow_hash: bool) -> bool {
    if allow_hash
        && hash_args(key)
            .iter()
            .any(|arg| norm_path(arg) == norm_path(lockfile))
    {
        return true;
    }
    earlier.iter().any(|step| shell_proves(step, key, lockfile))
}

fn shell_proves(step: &Step, key: &str, lockfile: &str) -> bool {
    let Some(id) = step
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return false;
    };
    let Some(run) = step.run.as_deref() else {
        return false;
    };
    proven_output_names(run, lockfile)
        .iter()
        .any(|name| has_fragment(key, &format!("steps.{id}.outputs.{name}")))
}

fn restore_present(field: Option<&Field>) -> bool {
    field.is_some_and(|field| field.value.lines().any(|line| !line.trim().is_empty()))
}

fn platform_message(path: &str, missing: &[String]) -> String {
    let list = missing
        .iter()
        .map(|part| format!("`{part}`"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Cache key for `{path}` is missing platform fragments {list}. Add each fragment as its own expression, such as ${{{{ runner.os }}}}. A different context such as matrix.os does not satisfy runner.os."
    )
}

fn digest_message(path: &str, lockfile: &str, key: &str, options: &Options) -> String {
    let hashed = hash_args(key).iter().any(|arg| norm_path(arg) == lockfile);
    if hashed && !options.allow_hash_files {
        return format!(
            "Cache key for `{path}` calls hashFiles for `{lockfile}`, but allowHashFiles is false. Set allowHashFiles to true or add a preceding run step that writes a hash of `{lockfile}` to the GITHUB_OUTPUT name referenced by the key."
        );
    }
    format!(
        "Cache key for `{path}` does not prove a digest of `{lockfile}`. Use hashFiles('{lockfile}') when allowHashFiles is true, or a preceding run step that writes a hash of `{lockfile}` to the GITHUB_OUTPUT name referenced by the key."
    )
}

fn missing_key(path: &str) -> String {
    format!(
        "Cache step for `{path}` has no key. Set key to include the configured platform fragments and a digest of the configured lockfile."
    )
}

fn restore_message(path: &str) -> String {
    format!(
        "Cache for `{path}` sets restore-keys. Remove restore-keys so a partial prefix cannot restore another platform or lockfile, or set rejectRestoreKeys to false."
    )
}

fn hit(file: &str, line: usize, message: String) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message,
        import: None,
        target: None,
    }
}
