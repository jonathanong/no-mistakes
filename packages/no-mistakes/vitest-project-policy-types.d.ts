/** Source literals supported by `vitest-project-policy`. */
export type VitestPropertyLiteral = string | number | boolean | null;

/** Literal syntax kinds checked without resolving computed expressions. */
export type VitestPropertyLiteralKind = "number" | "string" | "boolean";

/** Literal restrictions on every object in the configured source files. */
export interface VitestDeclaredProjectPolicy {
  forbiddenValues?: Record<string, VitestPropertyLiteral[]>;
  forbidLiteralKinds?: Record<string, VitestPropertyLiteralKind[]>;
}

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
  /** Helper paths or globs expanded against the prepared visible-file inventory. */
  helpers?: string[];
  /** Required scalar literals on each project's test options. */
  required?: Record<string, VitestPropertyLiteral>;
  /** Properties that must be absent from each project's test options. */
  forbidden?: string[];
  parity?: VitestPropertyParity[];
  declared?: VitestDeclaredProjectPolicy;
}
