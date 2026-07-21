import type { Relationship } from "./traversal-types";

export interface FlowOptions {
  target: string;
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  direction?: "deps" | "dependents" | "both";
  depth?: number;
  relationships?: Relationship[];
}

export interface FlowNode {
  id: string;
  kind: "file" | "symbol" | "module" | "queue-job";
  depth: number;
  file?: string;
  symbol?: string;
  module?: string;
  queueFile?: string;
  job?: string;
}

export interface FlowEdge {
  from: string;
  to: string;
  kind: string;
}

export interface FlowReport {
  root: string;
  target: string;
  nodes: FlowNode[];
  edges: FlowEdge[];
}

export interface FetchesOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  targets?: string[];
}
