// CI, impacted-checks, data-flow, infra, and Swift API declarations, split
// out of `index.d.ts` to stay under the 200-line file limit.

import type {
  CiEnvOptions,
  CiEnvReport,
  CiImpactOptions,
  CiImpactReport,
  CiTopologyOptions,
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
  WithInvocationOptions,
  WorkflowTopology,
  WorkflowTopologyIndex,
} from "./types";

export function lockfileDiff(
  options: WithInvocationOptions<LockfileDiffOptions>,
): Promise<LockfileDiffEntry[]>;
export function ciImpact(options: WithInvocationOptions<CiImpactOptions>): Promise<CiImpactReport>;
export function ciEnv(options: WithInvocationOptions<CiEnvOptions>): Promise<CiEnvReport>;
export function ciTopology(
  options?: WithInvocationOptions<CiTopologyOptions>,
): Promise<WorkflowTopology>;
/** Builds a query index over a `ciTopology()` result. Pure JS — never crosses the N-API boundary. */
export function createWorkflowTopologyIndex(topology: WorkflowTopology): WorkflowTopologyIndex;
export function impactedChecks(
  options: WithInvocationOptions<ImpactedChecksOptions>,
): Promise<ImpactedChecksReport>;
export function dataPw(options: WithInvocationOptions<DataPwOptions>): Promise<DataPwReport>;
export function effects(options: WithInvocationOptions<EffectsOptions>): Promise<EffectsReport>;
export function rscCallers(
  options: WithInvocationOptions<RscCallersOptions>,
): Promise<RscCallersReport>;
export function registryExtension(
  options: WithInvocationOptions<RegistryExtensionOptions>,
): Promise<RegistryExtensionReport>;
export function infraResourceRefs(
  options: WithInvocationOptions<InfraOptions & { address: string }>,
): Promise<ResourceRefRow[]>;
export function infraOutputs(
  options: WithInvocationOptions<InfraOptions & { moduleDir: string }>,
): Promise<ModuleOutputsResult>;
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
