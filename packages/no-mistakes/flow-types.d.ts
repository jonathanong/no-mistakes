import type { Relationship } from "./traversal-types";

export interface FlowOptions {
  target: string;
  root?: string;
  tsconfig?: string;
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
  root?: string;
  config?: string;
  targets?: string[];
}
