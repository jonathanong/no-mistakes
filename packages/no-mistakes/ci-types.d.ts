// Types for the `ci` and `impacted-checks` commands. Option interfaces use
// camelCase (deserialized with rename_all = "camelCase"); report interfaces use
// snake_case to match the serde-serialized output.

export interface CiImpactOptions {
  root?: string;
  config?: string;
  /** Changed file paths (relative to root or absolute). */
  files: string[];
}

export interface CiEnvOptions {
  root?: string;
  config?: string;
  /** Environment variable name (case-sensitive). */
  var: string;
}

export interface ImpactedChecksOptions {
  root?: string;
  config?: string;
  tsconfig?: string;
  base?: string;
  head?: string;
  changedFiles?: string[];
  changedFilesFile?: string;
  diff?: string;
}

export type TriggerMatch = "matched" | "always" | "not-matched" | "no-path-events";
export type PermissionLevel = "read" | "write" | "none";
export type PermissionSource = "job" | "workflow" | "default";

export interface ResolvedPermissions {
  source: PermissionSource;
  scopes: Record<string, PermissionLevel>;
  assumed_default: boolean;
}

export interface CiWarning {
  path: string;
  message: string;
}

export interface MatchedFilter {
  event: string;
  pattern: string;
}

export interface ImpactedJob {
  id: string;
  name?: string;
  uses?: string;
  permissions: ResolvedPermissions;
}

export interface ImpactedWorkflow {
  path: string;
  name?: string;
  trigger: TriggerMatch;
  reusable: boolean;
  matched_filters: MatchedFilter[];
  jobs: ImpactedJob[];
}

export interface CiImpactReport {
  changed_files: string[];
  workflows: ImpactedWorkflow[];
  warnings: CiWarning[];
}

export type EnvLocationKind = "definition" | "reference";
export type EnvScope = "workflow" | "job" | "step";

export interface CiEnvLocation {
  kind: EnvLocationKind;
  scope: EnvScope;
  job?: string;
  value?: string;
}

export interface CiEnvFile {
  path: string;
  locations: CiEnvLocation[];
}

export interface CiEnvReport {
  variable: string;
  files: CiEnvFile[];
  warnings: CiWarning[];
}

export type CheckKind = "test" | "generic";

export interface CheckCommand {
  name: string;
  kind: CheckKind;
  command: string[];
  files?: string[];
}

export interface ImpactedChecksReport {
  changed_files: string[];
  checks: CheckCommand[];
  warnings: Array<{ type: string; message: string; file: string }>;
  fallback_triggered: boolean;
}
