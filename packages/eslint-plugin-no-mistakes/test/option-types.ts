import plugin, {
  type NoTopLevelAwaitOptions,
  type TestNoTypedEnqueueFakeOptions,
  type TestNoUnfailableAssertionOptions,
  type TestNoUnfailableAssertionSeverity,
  type TestNoWallTimeAssertionsOptions,
  type NoLocaleLessDateFormatOptions,
  type PlaywrightTestTimeoutCapOptions,
  type PlaywrightNoBrowserNewPageOptions,
  type PlaywrightNoBrowserNewPageUnanalyzable,
  type PlaywrightAssertionTimeoutCapOptions,
  type PlaywrightNoHoistedUniqueTokenOptions,
  type PlaywrightRequireInteractiveTestIdOptions,
  type PlaywrightRuleUnanalyzable,
  type ReactNoNullishReactNodeOptions,
  type ReactNoUsePromiseResolveOptions,
  type NextjsNoManualScriptTagsOptions,
  type ServerRequireNullableFetchWrapperOptions,
  type RuleUnanalyzable,
  type PlaywrightNoRawScrollPaginationOptions,
  type PostgresNoExecutorAdapterMode,
  type PostgresNonWritingCallee,
  type PostgresNoExecutorAdapterOptions,
  type PostgresPoolRequiresErrorListenerOptions,
  type PostgresUnanalyzable,
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

export const postgresUnanalyzable: PostgresUnanalyzable = "advise";
export const options: PostgresNoExecutorAdapterOptions = {
  importSpecifier: "@example/db",
  executorTypeNames: ["QueryExecutor"],
  nonWritingCallees: [nonWritingCallee, namedCallee],
  mode,
  unanalyzable: postgresUnanalyzable,
};
export const invalidPostgresUnanalyzable: PostgresNoExecutorAdapterOptions = {
  // @ts-expect-error Unanalyzable policy must be advise, report, or ignore.
  unanalyzable: "warn",
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
export const styleClassRule = plugin.rules?.["test-no-style-class-assertion"];
export const commonJsStyleClassRule = pluginRequire.rules?.["test-no-style-class-assertion"];

export const poolListenerOptions: PostgresPoolRequiresErrorListenerOptions = {
  extraModules: ["@example/pg-compatible"],
  unanalyzable: "ignore",
};
export const invalidPoolUnanalyzable: PostgresPoolRequiresErrorListenerOptions = {
  // @ts-expect-error Unanalyzable policy must be advise, report, or ignore.
  unanalyzable: "warn",
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

export const unfailableOptions: TestNoUnfailableAssertionOptions = {
  severities: { "throwing-query": "error", "defined-then-deref": "off" },
  report: "error",
};
export const defaultUnfailableOptions: pluginRequire.TestNoUnfailableAssertionOptions = {};
// @ts-expect-error Severity must be error, advisory, or off.
export const invalidSeverity: TestNoUnfailableAssertionSeverity = "warn";

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

export const browserNewPageUnanalyzable: PlaywrightNoBrowserNewPageUnanalyzable = "finding";
export const browserNewPageOptions: PlaywrightNoBrowserNewPageOptions = {
  fixtureName: "appPage",
  browserFixtureName: "browser",
  registrationPackages: ["./fixtures"],
  unanalyzable: browserNewPageUnanalyzable,
  message: "Use the shared appPage fixture.",
};
export const defaultBrowserNewPageOptions: pluginRequire.PlaywrightNoBrowserNewPageOptions = {};
export const invalidBrowserNewPageUnanalyzable: PlaywrightNoBrowserNewPageOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};
export const invalidBrowserFixtureName: PlaywrightNoBrowserNewPageOptions = {
  // @ts-expect-error Fixture name must be a string.
  fixtureName: 42,
};

export const ruleUnanalyzable: PlaywrightRuleUnanalyzable = "ignore";
export const assertionTimeoutOptions: PlaywrightAssertionTimeoutCapOptions = {
  max: 10_000,
  unanalyzable: ruleUnanalyzable,
};
export const defaultAssertionTimeoutOptions: pluginRequire.PlaywrightAssertionTimeoutCapOptions =
  {};
export const invalidAssertionTimeoutOptions: PlaywrightAssertionTimeoutCapOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};

export const hoistedTokenOptions: PlaywrightNoHoistedUniqueTokenOptions = {
  tokenFactories: ["makeToken"],
  unanalyzable: "finding",
};
export const defaultHoistedTokenOptions: pluginRequire.PlaywrightNoHoistedUniqueTokenOptions = {};
export const invalidHoistedTokenOptions: PlaywrightNoHoistedUniqueTokenOptions = {
  // @ts-expect-error Token factories must be strings.
  tokenFactories: [false],
};

export const interactiveTestIdOptions: PlaywrightRequireInteractiveTestIdOptions = {
  selectorAttributes: ["data-pw"],
  interactiveComponents: ["Button"],
  slotProps: ["asChild"],
  unanalyzable: "finding",
};
export const defaultInteractiveTestIdOptions: pluginRequire.PlaywrightRequireInteractiveTestIdOptions =
  {};
export const invalidInteractiveTestIdOptions: PlaywrightRequireInteractiveTestIdOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};

export const unanalyzablePolicy: RuleUnanalyzable = "finding";
export const nullishOptions: ReactNoNullishReactNodeOptions = {
  unanalyzable: unanalyzablePolicy,
};
export const defaultNullishOptions: pluginRequire.ReactNoNullishReactNodeOptions = {};
export const invalidNullishOptions: ReactNoNullishReactNodeOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};

export const useResolveOptions: ReactNoUsePromiseResolveOptions = { unanalyzable: "ignore" };
export const invalidUseResolveOptions: ReactNoUsePromiseResolveOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};

export const scriptTagOptions: NextjsNoManualScriptTagsOptions = {
  allowInlineScriptIds: ["theme"],
  allowInlineScriptIdPatterns: ["^boot-"],
  structuredDataComponent: "StructuredData",
  unanalyzable: "finding",
};
export const invalidScriptTagOptions: NextjsNoManualScriptTagsOptions = {
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};

export const nullableFetchOptions: ServerRequireNullableFetchWrapperOptions = {
  getterCalleePatterns: ["^getWidget$"],
  requiredWrapperCallee: "wrapNullable",
  nullableReturnTypeNames: ["Widget"],
  unanalyzable: "finding",
};
export const invalidNullableFetchOptions: ServerRequireNullableFetchWrapperOptions = {
  getterCalleePatterns: ["^getWidget$"],
  requiredWrapperCallee: "wrapNullable",
  // @ts-expect-error Unanalyzable policy must be finding or ignore.
  unanalyzable: "warn",
};
