// Existing rule-policy assertions exclude only the independent hygiene audit.
// Whole-report parity assertions still compare all diagnostics unchanged.
function policyAdvisories(report) {
  return report.advisories.filter(
    (finding) =>
      finding.rule !== "unused-disable-directive" && finding.rule !== "invalid-disable-directive",
  );
}

module.exports = { policyAdvisories };
