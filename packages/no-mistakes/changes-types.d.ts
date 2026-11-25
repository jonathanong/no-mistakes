import type { WithInvocationOptions, WithInvocationResult } from "./invocation-types";

/** Changed fixture IDs mapped to RFC 6901 JSON Pointers. */
export type FixtureAttributePaths = Record<string, string[]>;

/** Stable semantic IDs; all lists are sorted and deduplicated. */
export interface KeyChanges {
  changed: string[];
  added: string[];
  removed: string[];
  /** Omitted for IDs that require whole-fixture selection. */
  attributePaths?: FixtureAttributePaths;
}

export type KeyedDiffFormat = "pnpm" | "npm" | "swift" | "nuget" | "json" | "strings" | "resx";

export interface KeyedDiffOptions {
  format: KeyedDiffFormat;
  /** Dotted object path or JSON pointer to an array. Use with idField. */
  array?: string;
  /** Nonempty string field used to identify each array entry. */
  idField?: string;
}

export type ChangeIdRule =
  | { fromPath: string; separator?: string; keysOf?: never }
  | { keysOf: KeyedDiffFormat | KeyedDiffOptions; fromPath?: never; separator?: never };

export interface ChangeRule {
  paths: string[];
  id?: ChangeIdRule;
}

export interface ChangeCategory {
  paths?: string[];
  id?: ChangeIdRule;
  rules?: ChangeRule[];
  exclude?: string[];
  /** Repository-relative test source globs searched through prepared sources. */
  tests?: string[];
  /** Literal match template with {id} or <id>; defaults to the ID itself. */
  match?: string;
  /** Set true only when the match loader returns the parsed fixture JSON unchanged. */
  loaderReturnsParsedJson?: boolean;
  /** Named framework full-suite trigger, or full-suite. Defaults to full-suite. */
  fallback?: string;
}

/** Top-level changes section of .no-mistakes.yml. */
export interface ChangesConfig {
  categories?: Record<string, ChangeCategory>;
}

export interface ChangesReportMetadata {
  schemaVersion: 1;
  complete: boolean;
  unclassified: string[];
  changedFiles?: string[];
  reasons?: string[];
}

/** Supply configured category names to obtain named category fields. */
export type ChangesReport<Categories extends string = never> = ChangesReportMetadata &
  Partial<Record<Categories, KeyChanges>>;

export interface ChangesDiffOptions {
  baseRoot: string;
  headRoot: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
}

/** Compare already-fetched local trees; incomplete roots return a structured report. */
export function changesDiff<Categories extends string = never>(
  options: WithInvocationOptions<ChangesDiffOptions>,
): Promise<WithInvocationResult<ChangesReport<Categories>>>;

/** Input artifacts may omit empty lists, but require at least one list per category. */
export type ChangeIdsInput = { attributePaths?: FixtureAttributePaths } & (
  | { changed: string[]; added?: string[]; removed?: string[] }
  | { changed?: string[]; added: string[]; removed?: string[] }
  | { changed?: string[]; added?: string[]; removed: string[] }
);

export type ChangesInputReport<Categories extends string = never> = ChangesReportMetadata &
  Partial<Record<Categories, ChangeIdsInput>>;
