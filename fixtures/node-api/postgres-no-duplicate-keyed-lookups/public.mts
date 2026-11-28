import {
  check,
  analyzeProject,
  type PostgresNoDuplicateKeyedLookupsOptions,
  type PostgresNoDuplicateKeyedLookupsUnanalyzableSql,
} from "no-mistakes";

const legacyModes: PostgresNoDuplicateKeyedLookupsUnanalyzableSql[] = ["fail", "ignore", "advise"];
const options: PostgresNoDuplicateKeyedLookupsOptions = {
  include: ["src/**/*.ts"],
  exclude: ["src/generated/**"],
  sqlInclude: ["queries/**/*.sql"],
  importSpecifier: "./db",
  executorNames: ["query"],
  excludeExecutorNames: [],
  executorFactoryNames: ["createDatabase"],
  executorTypeNames: ["Database"],
  executorOptionProperties: ["executor"],
  trustedSqlTags: [{ module: "./db", name: "sql" }],
  unanalyzable: "advise",
};
const additional: PostgresNoDuplicateKeyedLookupsOptions = {
  extraExecutorNames: ["execute"],
  unanalyzableSql: "ignore",
};
// @ts-expect-error Unknown option keys are rejected.
const invalid: PostgresNoDuplicateKeyedLookupsOptions = { unknownFlag: true };
const checkReport = check({ root: ".", config: ".no-mistakes.yml" });
const projectReport = analyzeProject({
  root: ".",
  config: ".no-mistakes.yml",
  reports: [{ type: "check" }],
});
void [legacyModes, options, additional, invalid, checkReport, projectReport];
