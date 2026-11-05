// Existing rule-policy assertions exclude only the independent hygiene audit.
// Whole-report parity assertions still compare all diagnostics unchanged.
function policyAdvisories(report) {
  return report.advisories.filter(
    (finding) =>
      finding.rule !== "unused-disable-directive" && finding.rule !== "invalid-disable-directive",
  );
}

function normalizeVerbatimWindowsPath(value) {
  if (typeof value !== "string") return value;
  if (value.startsWith("\\\\?\\UNC\\")) return "\\\\" + value.slice(8);
  return value.startsWith("\\\\?\\") ? value.slice(4) : value;
}

function normalizeCheckFailurePaths(report) {
  if (!Array.isArray(report?.incomplete)) return report;
  return {
    ...report,
    incomplete: report.incomplete.map((failure) => {
      const file = normalizeVerbatimWindowsPath(failure.file);
      const message =
        typeof failure.message === "string" &&
        typeof failure.file === "string" &&
        file !== failure.file
          ? failure.message.replaceAll(failure.file, file)
          : failure.message;
      return {
        ...failure,
        ...(file === failure.file ? {} : { file }),
        ...(message === failure.message ? {} : { message }),
      };
    }),
  };
}

module.exports = { normalizeCheckFailurePaths, normalizeVerbatimWindowsPath, policyAdvisories };
