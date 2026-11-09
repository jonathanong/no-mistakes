import {
  check,
  analyzeProject,
  type PostgresGroupingNotAggregationOptions,
  type PostgresGroupingNotAggregationCode,
} from "no-mistakes";

const codes: PostgresGroupingNotAggregationCode[] = [
  "all-of-candidate-pool",
  "distinct-over-unique-key",
  "distinct-over-union",
  "distinct-join-filter",
  "aggregate-existence-test",
];
const options: PostgresGroupingNotAggregationOptions = {
  schemaCatalogPath: "db/schema.json",
  include: ["src/**/*.ts"],
  exclude: ["src/generated/**"],
  sqlInclude: ["db/**/*.sql"],
  importSpecifier: "db",
  executorNames: ["query"],
  excludeExecutorNames: [],
  executorFactoryNames: ["createDatabase"],
  executorTypeNames: ["Database"],
  executorOptionProperties: ["executor"],
  sqlLiteralHelpers: ["literal"],
  trustedSqlTags: [{ module: "db", name: "sql" }],
};
const additional: PostgresGroupingNotAggregationOptions = { extraExecutorNames: ["execute"] };
// @ts-expect-error Grouping advice cannot be configured to fail.
const failing: PostgresGroupingNotAggregationOptions = { fail: true };
// @ts-expect-error Shape codes are a closed public contract.
const unknown: PostgresGroupingNotAggregationCode = "unknown";
const checkReport = check({ root: ".", config: ".no-mistakes.yml" });
const projectReport = analyzeProject({
  root: ".",
  config: ".no-mistakes.yml",
  reports: [{ type: "check" }],
});
void [codes, options, additional, failing, unknown, checkReport, projectReport];
