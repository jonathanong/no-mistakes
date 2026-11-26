import plugin, {
  type NoTopLevelAwaitOptions,
  type TestNoTypedEnqueueFakeOptions,
  type TestNoWallTimeAssertionsOptions,
  type NoLocaleLessDateFormatOptions,
  type PlaywrightTestTimeoutCapOptions,
  type PlaywrightNoRawScrollPaginationOptions,
  type PostgresNoExecutorAdapterMode,
  type PostgresNonWritingCallee,
  type PostgresNoExecutorAdapterOptions,
  type PostgresPoolRequiresErrorListenerOptions,
} from "../src/index.js";
import pluginRequire = require("../src/index.js");

export const mode: PostgresNoExecutorAdapterMode = "forwarding-only";
export const nonWritingCallee: PostgresNonWritingCallee = {
  module: "@example/assertions",
  export: "default",
};
export const namedCallee: pluginRequire.PostgresNonWritingCallee = {
  module: "@example/assertions",
  export: "requireValue",
};
// @ts-expect-error Exact import identities require an export name.
export const invalidCallee: PostgresNonWritingCallee = { module: "@example/assertions" };

export const options: PostgresNoExecutorAdapterOptions = {
  importSpecifier: "@example/db",
  executorTypeNames: ["QueryExecutor"],
  nonWritingCallees: [nonWritingCallee, namedCallee],
  mode,
};
export const dateFormatOptions: NoLocaleLessDateFormatOptions = {
  methods: ["toLocaleDateString"],
  constructors: ["Intl.DateTimeFormat"],
  allowUndefinedLocale: true,
  unknownValues: "finding",
};
export const defaultDateFormatOptions: pluginRequire.NoLocaleLessDateFormatOptions = {};

// @ts-expect-error Methods must be an array of strings.
export const invalidMethods: NoLocaleLessDateFormatOptions = { methods: [1] };
// @ts-expect-error Constructors must be an array of strings.
export const invalidConstructors: NoLocaleLessDateFormatOptions = { constructors: [1] };
export const invalidAllowUndefinedLocale: NoLocaleLessDateFormatOptions = {
  // @ts-expect-error The undefined-locale option must be boolean.
  allowUndefinedLocale: "true",
};
export const invalidUnknownLocaleValues: NoLocaleLessDateFormatOptions = {
  // @ts-expect-error Unknown value policy must be ignore or finding.
  unknownValues: "warn",
};

export const enqueueFakeOptions: TestNoTypedEnqueueFakeOptions = {
  singleExportPattern: "^enqueue[A-Z]",
  moduleGlobs: ["./queue"],
};
export const defaultEnqueueFakeOptions: pluginRequire.TestNoTypedEnqueueFakeOptions = {};
export const invalidEnqueueFakeOptions: TestNoTypedEnqueueFakeOptions = {
  // @ts-expect-error Module globs must be strings.
  moduleGlobs: [1],
};

export const rule = plugin.rules?.["postgres-no-executor-adapter"];
export const commonJsRule = pluginRequire.rules?.["postgres-no-executor-adapter"];

export const poolListenerOptions: PostgresPoolRequiresErrorListenerOptions = {
  extraModules: ["@example/pg-compatible"],
};
export const defaultPoolListenerOptions: pluginRequire.PostgresPoolRequiresErrorListenerOptions =
  {};
export const invalidPoolListenerOptions: PostgresPoolRequiresErrorListenerOptions = {
  // @ts-expect-error Additional modules must be strings.
  extraModules: [true],
};

// @ts-expect-error Unknown mode must fail at the declaration boundary.
export const invalid: PostgresNoExecutorAdapterMode = "quiet";

export const timeoutCapOptions: PlaywrightTestTimeoutCapOptions = {
  max: 30000,
  fixtureMax: 30000,
  unknownValues: "ignore",
  configUnknownValues: "inherit",
  registrationPackages: ["@example/widgets"],
  exportRoles: { "@example/widgets": { expect: "assertion" } },
  configFiles: ["playwright.config.ts"],
  banOverrides: true,
};
export const defaultTimeoutCapOptions: pluginRequire.PlaywrightTestTimeoutCapOptions = {};
export const invalidBanOverrides: PlaywrightTestTimeoutCapOptions = {
  // @ts-expect-error Override ban must be boolean.
  banOverrides: "yes",
};

export const awaitOptions: NoTopLevelAwaitOptions = {
  allowInImportMetaMain: false,
  message: "Initialize explicitly",
};
// @ts-expect-error Guard policy must be a boolean.
export const invalidAwaitOptions: NoTopLevelAwaitOptions = { allowInImportMetaMain: "yes" };

export const wallTimeOptions: TestNoWallTimeAssertionsOptions = { allowedPaths: ["benchmarks/**"] };
export const defaultWallTimeOptions: pluginRequire.TestNoWallTimeAssertionsOptions = {};
// @ts-expect-error Allowed benchmark paths must be strings.
export const invalidWallTimeOptions: TestNoWallTimeAssertionsOptions = { allowedPaths: [1] };

export const paginationOptions: PlaywrightNoRawScrollPaginationOptions = {
  cursorParams: ["after"],
  scrollHelper: "scrollToLoadMore",
  sentinelLocatorNames: ["sentinel"],
  sentinelTestIds: ["load-more"],
};
export const defaultPaginationOptions: pluginRequire.PlaywrightNoRawScrollPaginationOptions = {};
export const invalidSentinelTestIds: PlaywrightNoRawScrollPaginationOptions = {
  // @ts-expect-error Test IDs must be strings.
  sentinelTestIds: [true],
};
