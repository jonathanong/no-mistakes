import { read } from "@bench/db";

export const importedSql = "/* imported head */ SELECT 1";

export function buildSql() {
  // The helper's own executor is suppressed, but the caller remains a finding.
  // no-mistakes-disable-next-line postgres-require-query-annotation: helper-local only
  return read("SELECT 1"); // EXPECT_HELPER_LINE_SUPPRESSED
}
