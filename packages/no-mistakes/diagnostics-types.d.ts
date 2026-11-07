import type * as Api from "./index";

export interface DiagnosticsResult<T> {
  result: T;
  diagnostics: VerboseDiagnostics;
}

export interface VerboseDiagnostics {
  elapsedMs: number;
  timings: DiagnosticTiming[];
  work: Record<string, number>;
  phases: PhaseResourceDiagnostic[];
  hotFiles: HotFileDiagnostic[];
  peakRssKiB?: number;
}

export interface DiagnosticTiming {
  label: string;
  durationMs: number;
  parallel: boolean;
}

export interface PhaseResourceDiagnostic {
  label: string;
  wallMs: number;
  userMs: number;
  systemMs: number;
  utilization: number;
  peakRssKiB: number;
  peakRssDeltaKiB: number;
}

export type DiagnosticFilePass =
  | "parse"
  | "value_flow.extract"
  | "value_flow.finalize"
  | "postgres.annotation";

export interface HotFileDiagnostic {
  pass: DiagnosticFilePass;
  path: string;
  durationMs: number;
  units: number;
}

/** Analysis errors carry a partial report if observation began before rejection. */
export interface DiagnosticsError extends Error {
  diagnostics?: VerboseDiagnostics;
}

/** Supported public operations; arguments and results retain their existing API types. */
export interface DiagnosticOperations {
  analyzeProject: typeof Api.analyzeProject;
  analyzeTypeScriptModules: typeof Api.analyzeTypeScriptModules;
  callSites: typeof Api.callSites;
  check: typeof Api.check;
  ciEnv: typeof Api.ciEnv;
  ciImpact: typeof Api.ciImpact;
  ciTopology: typeof Api.ciTopology;
  ciTopologyImpact: typeof Api.ciTopologyImpact;
  dataPw: typeof Api.dataPw;
  deadExports: typeof Api.deadExports;
  dependencies: typeof Api.dependencies;
  dependents: typeof Api.dependents;
  effects: typeof Api.effects;
  exportsOf: typeof Api.exportsOf;
  extractExecutedPostgresSql: typeof Api.extractExecutedPostgresSql;
  fetches: typeof Api.fetches;
  flow: typeof Api.flow;
  generatePostgresCatalog: typeof Api.generatePostgresCatalog;
  impactedChecks: typeof Api.impactedChecks;
  importUsages: typeof Api.importUsages;
  importers: typeof Api.importers;
  infraOutputs: typeof Api.infraOutputs;
  infraResourceRefs: typeof Api.infraResourceRefs;
  infraTestFor: typeof Api.infraTestFor;
  lockStatus: typeof Api.lockStatus;
  lockfileDiff: typeof Api.lockfileDiff;
  parsePostgresSql: typeof Api.parsePostgresSql;
  playwrightCheck: typeof Api.playwrightCheck;
  playwrightEdges: typeof Api.playwrightEdges;
  playwrightRelated: typeof Api.playwrightRelated;
  playwrightTests: typeof Api.playwrightTests;
  queueCheck: typeof Api.queueCheck;
  queueEdges: typeof Api.queueEdges;
  queueRelated: typeof Api.queueRelated;
  queues: typeof Api.queues;
  reactAnalyze: typeof Api.reactAnalyze;
  reactCheck: typeof Api.reactCheck;
  reactUsages: typeof Api.reactUsages;
  registryExtension: typeof Api.registryExtension;
  related: typeof Api.related;
  resolveCheck: typeof Api.resolveCheck;
  resolveConfig: typeof Api.resolveConfig;
  rscCallers: typeof Api.rscCallers;
  serverContracts: typeof Api.serverContracts;
  serverRouteEdges: typeof Api.serverRouteEdges;
  serverRouteList: typeof Api.serverRouteList;
  serverRouteRelated: typeof Api.serverRouteRelated;
  serverRoutes: typeof Api.serverRoutes;
  swiftImporters: typeof Api.swiftImporters;
  swiftTestTargets: typeof Api.swiftTestTargets;
  symbols: typeof Api.symbols;
  testsAudit: typeof Api.testsAudit;
  testsGraph: typeof Api.testsGraph;
  testsImpact: typeof Api.testsImpact;
  testsPlan: typeof Api.testsPlan;
  testsTargets: typeof Api.testsTargets;
  testsWhy: typeof Api.testsWhy;
  validateMermaidMarkdown: typeof Api.validateMermaidMarkdown;
}
