/**
 * Literal per-package Swift runner argv from `tests.swift.baseCommands`.
 * Keys are declared `tests.swift.packages` paths. Omitted packages use
 * `["swift", "test"]`.
 */
export interface SwiftPackageBaseCommands {
  [packageRoot: string]: string[];
}
