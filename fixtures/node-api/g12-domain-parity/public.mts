import {
  ciTopology,
  generatePostgresCatalog,
  lockfileDiff,
  type CiTopologyMermaidOptions,
  type LockfileDiffEntry,
  type LockfileDiffReport,
  type LockfileDiffReportOptions,
  type LockfileDiffWarning,
  type WorkflowTopology,
} from "no-mistakes";

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

void [diagram, topology, warning, entries, catalog];
