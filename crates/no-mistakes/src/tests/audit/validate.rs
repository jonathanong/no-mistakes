use super::*;
use anyhow::bail;

pub(super) fn artifacts(
    plan: &TestAuditPlanArtifact,
    observations: &TestAuditObservationsArtifact,
) -> Result<()> {
    if plan.schema_version != 1 || observations.schema_version != 1 {
        bail!("tests audit supports schema_version 1 artifacts only");
    }
    validate_provenance(&plan.provenance, &observations.provenance)?;
    validate_run_shape(observations)?;
    validate_changed_scope(plan)?;
    validate_test_files(plan, observations)
}

fn validate_provenance(
    plan: &TestAuditProvenance,
    observations: &TestAuditProvenance,
) -> Result<()> {
    for provenance in [plan, observations] {
        hex(
            &provenance.checkout_revision,
            &[40, 64],
            "checkout_revision",
        )?;
        hex(&provenance.source_digest, &[64], "source_digest")?;
        hex(&provenance.scope_digest, &[64], "scope_digest")?;
    }
    if plan != observations {
        bail!("Audit provenance mismatch: checkout_revision, source_digest, and scope_digest must match exactly; regenerate both artifacts from the same source and runner scope");
    }
    Ok(())
}

fn validate_run_shape(observations: &TestAuditObservationsArtifact) -> Result<()> {
    if observations.granularity != "per-test-file"
        || observations.suite != "full"
        || !observations.complete
    {
        bail!("Audit observations require granularity 'per-test-file', suite 'full', and complete true; aggregate coverage and targeted/incomplete runs are unsupported");
    }
    Ok(())
}

fn validate_changed_scope(plan: &TestAuditPlanArtifact) -> Result<()> {
    let changes = paths(&plan.changed_files)?;
    if changes.is_empty() {
        bail!("Audit changed_files must explicitly identify at least one changed file");
    }
    let embedded = paths(&plan.plan.changed_files)?;
    if !embedded.is_empty() && embedded != changes {
        bail!("Audit changed_files must match the embedded TestPlan changed_files inventory");
    }
    let mut symbols = BTreeSet::new();
    for symbol in &plan.changed_symbols {
        validate_symbol(symbol)?;
        if !changes.contains(symbol.file.as_str()) || !symbols.insert(symbol) {
            bail!("Audit changed_symbols must be unique and belong to changed_files");
        }
    }
    Ok(())
}

fn validate_test_files(
    plan: &TestAuditPlanArtifact,
    observations: &TestAuditObservationsArtifact,
) -> Result<()> {
    let mut selected = BTreeSet::new();
    for test in &plan.plan.selected_tests {
        path(&test.test_file)?;
        if !selected.insert(&test.test_file) {
            bail!("Duplicate selected test_file: {}", test.test_file);
        }
    }
    let mut tests = BTreeSet::new();
    for test in &observations.tests {
        path(&test.test_file)?;
        if !tests.insert(&test.test_file) {
            bail!("Duplicate observed test_file: {}; merge case traces into one file record before auditing", test.test_file);
        }
        let files = paths(&test.executed_files)?;
        for symbol in &test.executed_symbols {
            validate_symbol(symbol)?;
            if !files.contains(symbol.file.as_str()) {
                bail!(
                    "Observed symbol file {} must also appear in executed_files",
                    symbol.file
                );
            }
        }
    }
    Ok(())
}

fn hex(value: &str, lengths: &[usize], field: &str) -> Result<()> {
    if !lengths.contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("Audit {field} must be a lowercase hexadecimal identity of the documented length");
    }
    Ok(())
}

fn paths(files: &[String]) -> Result<BTreeSet<&str>> {
    files
        .iter()
        .map(|file| {
            path(file)?;
            Ok(file.as_str())
        })
        .collect()
}

fn path(file: &str) -> Result<()> {
    let bytes = file.as_bytes();
    let drive_prefix =
        bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':');
    if file.contains('\\')
        || drive_prefix
        || file.chars().any(char::is_control)
        || file
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        bail!("Audit path `{file}` must be a normalized root-relative slash path without '.', '..', drive prefixes, or empty components");
    }
    Ok(())
}

fn validate_symbol(symbol: &TestAuditSymbol) -> Result<()> {
    path(&symbol.file)?;
    if symbol.symbol.trim().is_empty() || symbol.symbol.chars().any(char::is_control) {
        bail!("Audit symbol must be a nonempty exact symbol name without control characters");
    }
    Ok(())
}
