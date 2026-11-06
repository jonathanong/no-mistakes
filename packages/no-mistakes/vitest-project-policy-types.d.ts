/** Source literals supported by `vitest-project-policy`. */
export type VitestPropertyLiteral = string | number | boolean | null;

/** Paired declarations whose corresponding values agree across projects. */
export interface VitestPropertyParity {
  properties: [string, string];
  /** Compare corresponding values to the root test object instead of the first project. */
  compareRoot?: boolean;
}

/** Options for the source-only `vitest-project-policy` check rule. */
export interface VitestProjectPolicyOptions {
  /** Explicit root config path relative to the repository root. */
  config: string;
  /** Finite helper source paths; imports outside this list remain incomplete. */
  helpers?: string[];
  /** Required scalar literals on each project's test options. */
  required?: Record<string, VitestPropertyLiteral>;
  /** Properties that must be absent from each project's test options. */
  forbidden?: string[];
  parity?: VitestPropertyParity[];
}
