import type { Relationship } from "./traversal-types";

/** File-target allowances applied only to matching selected origin files. */
export interface ForbiddenDependencyException {
  /** Exact files, directories, or repository-relative globs selecting origins. */
  roots: string[];
  /** Repository-relative file globs; these do not stop dependency traversal. */
  allowedFiles: string[];
}

/** Options for a `forbidden-dependencies` rule application in the config file. */
export interface ForbiddenDependenciesOptions {
  /** Exact files, directories, or repository-relative globs selecting origins. */
  roots: string[];
  /** Matching origins are removed without pruning dependency traversal. */
  excludeRoots?: string[];
  /** Package or module-specifier globs. */
  forbiddenModules?: string[];
  /** Repository-relative file globs. */
  forbiddenFiles?: string[];
  /** Relationship filters; omission follows all relationships. */
  relationships?: Relationship[];
  /** Matching exceptions union their file allowances for each selected origin. */
  exceptions?: ForbiddenDependencyException[];
}
