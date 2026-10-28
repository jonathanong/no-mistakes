import type { TypeScriptModulesOptions, TypeScriptModulesReport } from "./typescript-module-types";
import type {
  PostgresCatalog,
  PostgresCatalogOptions,
  PostgresCompleteCatalog,
  PostgresOrderingCatalog,
} from "./postgres-catalog-types";
// CI, impacted-checks, data-flow, infra, and Swift API declarations, split
// out of `index.d.ts` to stay under the 200-line file limit.

import type {
  CiEnvOptions,
  CiEnvReport,
  CiImpactOptions,
  CiImpactReport,
  CiTopologyOptions,
  CiTopologyImpactOptions,
  CiTopologyImpactReport,
  DataPwOptions,
  DataPwReport,
  EffectsOptions,
  EffectsReport,
  ImpactedChecksOptions,
  ImpactedChecksReport,
  InfraOptions,
  LockfileDiffEntry,
  LockfileDiffOptions,
  ModuleOutputsResult,
  RegistryExtensionOptions,
  RegistryExtensionReport,
  ResourceRefRow,
  RscCallersOptions,
  RscCallersReport,
  SwiftImporterRow,
  SwiftOptions,
  SwiftTestTargetRow,
  TestForRow,
  TestAuditReport,
  TestsAuditOptions,
  WithInvocationOptions,
  WithInvocationResult,
  WorkflowTopology,
  WorkflowTopologyIndex,
} from "./types";

export function lockfileDiff(
  options: WithInvocationOptions<LockfileDiffOptions>,
): Promise<LockfileDiffEntry[]>;
export function ciImpact(
  options: WithInvocationOptions<CiImpactOptions>,
): Promise<WithInvocationResult<CiImpactReport>>;
export function ciEnv(
  options: WithInvocationOptions<CiEnvOptions>,
): Promise<WithInvocationResult<CiEnvReport>>;
export function ciTopology(
  options?: WithInvocationOptions<CiTopologyOptions>,
): Promise<WithInvocationResult<WorkflowTopology>>;
export function ciTopologyImpact(
  options: WithInvocationOptions<CiTopologyImpactOptions>,
): Promise<WithInvocationResult<CiTopologyImpactReport>>;
/** Builds a query index over a `ciTopology()` result. Pure JS — never crosses the N-API boundary. */
export function createWorkflowTopologyIndex(topology: WorkflowTopology): WorkflowTopologyIndex;
export function impactedChecks(
  options: WithInvocationOptions<ImpactedChecksOptions>,
): Promise<WithInvocationResult<ImpactedChecksReport>>;
export function dataPw(
  options: WithInvocationOptions<DataPwOptions>,
): Promise<WithInvocationResult<DataPwReport>>;
export function effects(
  options: WithInvocationOptions<EffectsOptions>,
): Promise<WithInvocationResult<EffectsReport>>;
export function rscCallers(
  options: WithInvocationOptions<RscCallersOptions>,
): Promise<WithInvocationResult<RscCallersReport>>;
export function registryExtension(
  options: WithInvocationOptions<RegistryExtensionOptions>,
): Promise<WithInvocationResult<RegistryExtensionReport>>;
export function infraResourceRefs(
  options: WithInvocationOptions<InfraOptions & { address: string }>,
): Promise<ResourceRefRow[]>;
export function infraOutputs(
  options: WithInvocationOptions<InfraOptions & { moduleDir: string }>,
): Promise<WithInvocationResult<ModuleOutputsResult>>;
export function infraTestFor(
  options: WithInvocationOptions<InfraOptions & { tfFile: string }>,
): Promise<TestForRow[]>;
export function swiftImporters(
  options: WithInvocationOptions<SwiftOptions & { file: string }>,
): Promise<SwiftImporterRow[]>;
export function swiftTestTargets(
  options: WithInvocationOptions<SwiftOptions & { file: string }>,
): Promise<SwiftTestTargetRow[]>;
export function version(): Promise<string>;

export function testsAudit(
  options: WithInvocationOptions<TestsAuditOptions>,
): Promise<WithInvocationResult<TestAuditReport>>;

/**
 * Observe committed PostgreSQL metadata in a read-only transaction and return the catalog that
 * `schemaCatalogPath` reads. Requires psql. `coverage: "ordering"` returns only the ordering facts.
 */
export function generatePostgresCatalog(
  options: WithInvocationOptions<PostgresCatalogOptions & { coverage: "ordering" }>,
): Promise<WithInvocationResult<PostgresOrderingCatalog>>;
export function generatePostgresCatalog(
  options: WithInvocationOptions<PostgresCatalogOptions & { coverage?: "complete" }>,
): Promise<WithInvocationResult<PostgresCompleteCatalog>>;
export function generatePostgresCatalog(
  options: WithInvocationOptions<PostgresCatalogOptions>,
): Promise<WithInvocationResult<PostgresCatalog>>;

export function analyzeTypeScriptModules(
  options: WithInvocationOptions<TypeScriptModulesOptions>,
): Promise<WithInvocationResult<TypeScriptModulesReport>>;

/** Read-only report of the machine-wide invocation lock; never waits and takes no lock. */
export function lockStatus(): Promise<import("./invocation-types").LockStatus>;
