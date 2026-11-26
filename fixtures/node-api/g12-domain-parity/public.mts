import {
  ciTopology,
  generatePostgresCatalog,
  lockfileDiff,
  type CiTopologyMermaidOptions,
  type LockfileDiffEntry,
  type LockfileDiffReport,
  type LockfileDiffReportOptions,
  type LockfileDiffWarning,
  type PostgresBoundedStatementKind,
  type PostgresBoundedStatementsOptions,
  type PostgresLegacyUnanalyzableSql,
  type PostgresNoMutableSqlRegistryOptions,
  type PostgresNoUnboundedSqlIdentifierOptions,
  type WorkflowTopology,
} from "no-mistakes";

const statements: PostgresBoundedStatementKind[] = ["select", "update", "delete"];
const legacy: PostgresLegacyUnanalyzableSql = "fail";
const bounded: PostgresBoundedStatementsOptions = {
  schemaCatalogPath: "catalog.json",
  statements,
  unanalyzableSql: legacy,
};
const registry: PostgresNoMutableSqlRegistryOptions = { unanalyzableSql: legacy };
const identifier: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzableSql: legacy };
const modern: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzable: "report" };

// @ts-expect-error The modern policy spells this "report".
const invalidModern: PostgresNoUnboundedSqlIdentifierOptions = { unanalyzable: "fail" };
// @ts-expect-error The legacy policy spells this "fail".
const invalidLegacy: PostgresBoundedStatementsOptions = { ...bounded, unanalyzableSql: "report" };
// @ts-expect-error INSERT is outside the bounded-statements selector.
const invalidStatement: PostgresBoundedStatementKind = "insert";

const diagramOptions: CiTopologyMermaidOptions = { format: "mermaid" };
const diagram: string = await ciTopology(diagramOptions);
const topology: WorkflowTopology = await ciTopology({ format: "json" });
const reportOptions: LockfileDiffReportOptions = {
  base: "HEAD",
  lockfile: "bun.lockb",
  includeWarnings: true,
};
const report: LockfileDiffReport = await lockfileDiff(reportOptions);
const warning: LockfileDiffWarning | undefined = report.warnings[0];
const entries: LockfileDiffEntry[] = await lockfileDiff({ base: "HEAD", includeWarnings: false });
const catalog = await generatePostgresCatalog({
  connectionEnv: "SAMPLE_DB_URL",
  schema: "public",
  output: "catalog.json",
});

void [registry, identifier, modern, invalidModern, invalidLegacy, invalidStatement];
void [diagram, topology, warning, entries, catalog];
