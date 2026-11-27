import type {
  ModuleMockBoundaryOptions,
  ModuleMockPreserveExportsOptions,
  NoVitestSequentialOptions,
  RuleUnanalyzable,
  TestNoDelayedRejectsOptions,
  TestNoErrorMessageMatchingOptions,
  TestNoFixedTempPathsOptions,
  TestNoSharedStateOptions,
  TestNoSkipModifier,
  TestNoSkipsOptions,
  TestNoTypedEnqueueFakeOptions,
  TestNoWallTimeAssertionsOptions,
  VitestMockTestFileNamingOptions,
} from "../src/index.js";

export const policy: RuleUnanalyzable = "ignore";
// @ts-expect-error Unanalyzable policy must be finding or ignore.
export const invalidPolicy: RuleUnanalyzable = "warn";

export const sequentialOptions: NoVitestSequentialOptions = { unanalyzable: policy };
export const namingOptions: VitestMockTestFileNamingOptions = { unanalyzable: policy };
export const boundaryOptions: ModuleMockBoundaryOptions = {
  unanalyzable: policy,
  internalSpecifiers: ["@app/**"],
  includePathPatterns: ["test/**"],
  excludePathPatterns: ["dist/**"],
  requireLiteralSpecifiers: true,
  baseline: [["file.ts", "./mod", 1]],
  integrationExports: {
    specifiers: ["@app/**"],
    sourcePathTemplates: ["src/{specifierSuffix}.ts"],
    specifierPrefix: "@app/",
    extensions: [".ts"],
    reexportExtensions: [".ts"],
    markerRegex: "marker",
  },
};
export const preserveOptions: ModuleMockPreserveExportsOptions = {
  unanalyzable: policy,
  internalSpecifiers: ["@app/**"],
  includePathPatterns: ["test/**"],
  excludePathPatterns: ["dist/**"],
  baseline: [["file.ts", "./mod"]],
};
export const invalidBoundaryBaseline: ModuleMockBoundaryOptions = {
  // @ts-expect-error Boundary baseline entries are file, specifier, and count.
  baseline: [["file.ts", "./mod"]],
};
export const delayedOptions: TestNoDelayedRejectsOptions = { unanalyzable: policy };
export const messageOptions: TestNoErrorMessageMatchingOptions = { unanalyzable: policy };
export const tempOptions: TestNoFixedTempPathsOptions = { unanalyzable: policy };
export const sharedOptions: TestNoSharedStateOptions = {
  allowBeforeAllAssignments: true,
  unanalyzable: policy,
};
export const skipModifier: TestNoSkipModifier = "todo";
export const skipOptions: TestNoSkipsOptions = {
  allow: [skipModifier],
  testModules: ["./custom-test"],
  unanalyzable: policy,
};
export const wallOptions: TestNoWallTimeAssertionsOptions = {
  allowedPaths: ["benchmarks/**"],
  unanalyzable: policy,
};
export const enqueueOptions: TestNoTypedEnqueueFakeOptions = {
  singleExportPattern: "^enqueue$",
  moduleGlobs: ["./queue"],
  unanalyzable: policy,
};
export const invalidSkip: TestNoSkipsOptions = {
  // @ts-expect-error Skip modifiers are a fixed set.
  allow: ["pending"],
};
