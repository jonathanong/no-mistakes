import type { ImpactEdgeDetail, TestPlan } from "./test-types";

export interface TestsWhyOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  test: string;
  changed?: string;
  plan?: string;
  planJson?: SavedTestPlan | string;
}

export interface WhyStep {
  node: string;
  via?: string | null;
  detail?: ImpactEdgeDetail | null;
}

/** A current or pre-`changedFiles` plan accepted by saved-plan document APIs. */
export type SavedTestPlan = Omit<TestPlan, "changedFiles"> & {
  changedFiles?: string[];
};

export interface TestsPlanDocumentOptions {
  plan?: string;
  planJson?: SavedTestPlan | string;
}

export interface TestGraph {
  nodes: Array<{ name: string; type: "changed" | "test" | "intermediate" }>;
  edges: Array<{ from: string; to: string; via: string; detail?: ImpactEdgeDetail }>;
}
