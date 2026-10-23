const fs = require("node:fs");
const path = require("node:path");

const LIMIT_MS = 5_000;
const TOP_COUNT = 10;

function summarize(report) {
  if (!report || !Array.isArray(report.testResults) || report.testResults.length === 0) {
    throw new Error("Vitest JSON report has no testResults");
  }

  const rows = [];
  for (const file of report.testResults) {
    if (typeof file.name !== "string" || !Array.isArray(file.assertionResults)) {
      throw new Error("Vitest JSON report has an invalid test result");
    }
    for (const test of file.assertionResults) {
      if (
        typeof test.fullName !== "string" ||
        !["passed", "failed", "pending", "skipped", "todo"].includes(test.status)
      ) {
        throw new Error(`Vitest JSON report has an invalid assertion in ${file.name}`);
      }
      const completed = test.status === "passed" || test.status === "failed";
      if (
        completed &&
        (typeof test.duration !== "number" || !Number.isFinite(test.duration) || test.duration < 0)
      ) {
        throw new Error(`Completed test has no finite duration: ${file.name} > ${test.fullName}`);
      }
      if (
        test.duration != null &&
        (typeof test.duration !== "number" || !Number.isFinite(test.duration) || test.duration < 0)
      ) {
        throw new Error(`Test has an invalid duration: ${file.name} > ${test.fullName}`);
      }
      rows.push({
        file: file.name,
        name: test.fullName,
        status: test.status,
        durationMs: test.duration ?? null,
      });
    }
  }
  if (
    rows.length === 0 ||
    !rows.some((row) => row.status === "passed" || row.status === "failed")
  ) {
    throw new Error("Vitest JSON report has no completed tests");
  }
  if (Number.isInteger(report.numTotalTests) && report.numTotalTests !== rows.length) {
    throw new Error(
      `Vitest JSON test count mismatch: ${report.numTotalTests} reported, ${rows.length} listed`,
    );
  }
  const completed = rows.filter((row) => row.status === "passed" || row.status === "failed");
  const slowPassed = rows.filter((row) => row.status === "passed" && row.durationMs >= LIMIT_MS);
  const ranked = [...completed].sort(
    (a, b) =>
      b.durationMs - a.durationMs || a.file.localeCompare(b.file) || a.name.localeCompare(b.name),
  );
  return { rows, completed, slowPassed, ranked };
}

function csvCell(value) {
  const string = value == null ? "" : String(value);
  return `"${string.replaceAll('"', '""')}"`;
}

function csv(rows) {
  return (
    [
      "file,test,status,duration_ms",
      ...rows.map((row) => [row.file, row.name, row.status, row.durationMs].map(csvCell).join(",")),
    ].join("\n") + "\n"
  );
}

function markdown(summary) {
  const counts = Object.groupBy(summary.rows, (row) => row.status);
  const lines = [
    "### Real N-API test timing",
    "",
    `Completed: ${summary.completed.length}; passed: ${counts.passed?.length ?? 0}; failed: ${counts.failed?.length ?? 0}; skipped: ${counts.skipped?.length ?? 0}; pending: ${counts.pending?.length ?? 0}; todo: ${counts.todo?.length ?? 0}.`,
    `Passed tests at or above ${LIMIT_MS} ms: ${summary.slowPassed.length}.`,
    "",
    "| Duration (ms) | Status | Test | File |",
    "| ---: | --- | --- | --- |",
  ];
  for (const row of summary.ranked.slice(0, TOP_COUNT)) {
    const safe = (value) => String(value).replaceAll("|", "\\|").replaceAll("\n", " ");
    lines.push(
      `| ${row.durationMs.toFixed(1)} | ${row.status} | ${safe(row.name)} | ${safe(row.file)} |`,
    );
  }
  return lines.join("\n") + "\n";
}

function main(args = process.argv.slice(2)) {
  const [input, outputDir] = args;
  if (!input || !outputDir)
    throw new Error("usage: report-native-api-tests.cjs <vitest-json> <output-dir>");
  const report = JSON.parse(fs.readFileSync(input, "utf8"));
  const summary = summarize(report);
  const md = markdown(summary);
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(path.join(outputDir, "native-api-tests-durations.csv"), csv(summary.rows));
  fs.writeFileSync(path.join(outputDir, "native-api-tests-summary.md"), md);
  process.stdout.write(md);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, md);
  if (summary.slowPassed.length > 0) {
    throw new Error(
      `${summary.slowPassed.length} passed real N-API test(s) reached ${LIMIT_MS} ms`,
    );
  }
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 1;
  }
}

module.exports = { summarize, csv, markdown, main };
