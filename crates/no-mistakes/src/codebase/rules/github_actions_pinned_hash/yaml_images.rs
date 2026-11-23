use super::{RuleFinding, RULE_ID};
use crate::codebase::rules::github_actions_workflow_files::job_lines::document::{Document, Kind};

pub(super) fn append_images(content: &str, file: &str, findings: &mut Vec<RuleFinding>) {
    let Some(document) = Document::parse_with_scalars(content) else {
        return;
    };
    for &root in document.roots() {
        if let Some(jobs) = document
            .field(root, "jobs")
            .and_then(|id| document.mapping(id))
        {
            for job in jobs.as_chunks::<2>().0 {
                inspect_job(&document, job[1], file, findings);
            }
        } else if let Some(steps) = document.sequence(root) {
            inspect_steps(&document, steps, file, findings);
        } else if let Some(steps) = document
            .field(root, "runs")
            .and_then(|runs| document.field(runs, "steps"))
            .and_then(|steps| document.sequence(steps))
        {
            inspect_steps(&document, steps, file, findings);
        }
    }
}

fn inspect_job(document: &Document, job: usize, file: &str, findings: &mut Vec<RuleFinding>) {
    if document.mapping(job).is_none() {
        return;
    }
    inspect_docker_use(document, job, file, findings);
    if let Some(container) = document.field(job, "container") {
        if document.mapping(container).is_some() {
            inspect_field(document, container, "image", file, findings);
        } else {
            inspect_field(document, job, "container", file, findings);
        }
    }
    if let Some(services) = document
        .field(job, "services")
        .and_then(|id| document.mapping(id))
    {
        for service in services.as_chunks::<2>().0 {
            inspect_field(document, service[1], "image", file, findings);
        }
    }
    if let Some(steps) = document
        .field(job, "steps")
        .and_then(|id| document.sequence(id))
    {
        inspect_steps(document, steps, file, findings);
    }
}

fn inspect_steps(
    document: &Document,
    steps: &[usize],
    file: &str,
    findings: &mut Vec<RuleFinding>,
) {
    for &step in steps {
        inspect_docker_use(document, step, file, findings);
    }
}

fn inspect_docker_use(
    document: &Document,
    parent: usize,
    file: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(image) = document
        .field(parent, "uses")
        .and_then(|value| document.scalar(value))
        .and_then(|uses| uses.trim().strip_prefix("docker://"))
    else {
        return;
    };
    report(document, parent, "uses", image, file, findings);
}

fn inspect_field(
    document: &Document,
    parent: usize,
    key: &str,
    file: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let Some(value) = document.field(parent, key) else {
        return;
    };
    if let Some(image) = document.scalar(value) {
        report(document, parent, key, image, file, findings);
    }
}

fn report(
    document: &Document,
    parent: usize,
    key: &str,
    image: &str,
    file: &str,
    findings: &mut Vec<RuleFinding>,
) {
    let image = image.trim();
    let expression = image.contains("${{");
    if !expression && pinned(image) {
        return;
    }
    let line = if matches!(document.nodes[parent].kind, Kind::Alias(_)) {
        document.nodes[parent].line
    } else {
        document
            .field_line(parent, key)
            .unwrap_or(document.nodes[parent].line)
    };
    findings.push(RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line,
        message: if expression {
            format!("{file}:{line}: `{key}` image is an expression; cannot verify its digest")
        } else {
            format!(
                "{file}:{line}: `{key}` image `{image}` must be pinned with `@sha256:<64-hex-digest>`; resolve and pin the image digest"
            )
        },
        import: None,
        target: expression.then(|| "unanalyzable".to_owned()),
    });
}

fn pinned(image: &str) -> bool {
    let Some((name, digest)) = image.rsplit_once("@sha256:") else {
        return false;
    };
    !name.is_empty() && digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
}
