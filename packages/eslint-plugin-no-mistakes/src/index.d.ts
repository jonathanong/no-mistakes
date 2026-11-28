import type { ESLint } from "eslint";

declare const plugin: ESLint.Plugin;
declare namespace plugin {
  export interface NoTopLevelAwaitOptions {
    allowInImportMetaMain?: boolean;
    message?: string;
  }

  export type PlaywrightTimeoutExportRole = "assertion" | "ordinary" | "opaque";

  export interface PlaywrightTestTimeoutCapOptions {
    max?: number;
    fixtureMax?: number;
    unknownValues?: "ignore" | "finding";
    configUnknownValues?: "inherit" | "defer-to-runner";
    registrationPackages?: string[];
    exportRoles?: Record<string, Record<string, PlaywrightTimeoutExportRole>>;
    configFiles?: string[];
    banOverrides?: boolean;
  }

  export type PlaywrightRuleUnanalyzable = "finding" | "ignore";

  /** @deprecated Use PlaywrightRuleUnanalyzable. Removed in 0.111.0. */
  export type PlaywrightNoBrowserNewPageUnanalyzable = PlaywrightRuleUnanalyzable;

  export type RuleUnanalyzable = "finding" | "ignore";

  export interface PlaywrightNoBrowserNewPageOptions {
    fixtureName?: string;
    browserFixtureName?: string;
    registrationPackages?: string[];
    unanalyzable?: PlaywrightRuleUnanalyzable;
    message?: string;
  }

  export interface PlaywrightAssertionTimeoutCapOptions {
    max?: number;
    unanalyzable?: PlaywrightRuleUnanalyzable;
  }

  export interface PlaywrightNoHoistedUniqueTokenOptions {
    tokenFactories?: string[];
    unanalyzable?: PlaywrightRuleUnanalyzable;
  }

  export interface PlaywrightRequireInteractiveTestIdOptions {
    selectorAttributes?: string[];
    interactiveComponents?: string[];
    slotProps?: string[];
    unanalyzable?: PlaywrightRuleUnanalyzable;
  }

  export interface PlaywrightNoRawScrollPaginationOptions {
    cursorParams?: string[];
    scrollHelper?: string;
    sentinelLocatorNames?: string[];
    sentinelTestIds?: string[];
  }

  export interface NoVitestSequentialOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface VitestMockTestFileNamingOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface ModuleMockIntegrationExports {
    specifiers?: string[];
    sourcePathTemplates?: string[];
    specifierPrefix?: string;
    extensions?: string[];
    reexportExtensions?: string[];
    markerRegex?: string;
  }

  export interface ModuleMockBoundaryOptions {
    internalSpecifiers?: string[];
    includePathPatterns?: string[];
    excludePathPatterns?: string[];
    requireLiteralSpecifiers?: boolean;
    baseline?: [string, string, number][];
    integrationExports?: ModuleMockIntegrationExports;
    unanalyzable?: RuleUnanalyzable;
  }

  export interface ModuleMockPreserveExportsOptions {
    internalSpecifiers?: string[];
    includePathPatterns?: string[];
    excludePathPatterns?: string[];
    baseline?: [string, string][];
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoDelayedRejectsOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoErrorMessageMatchingOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoFixedTempPathsOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoSharedStateOptions {
    allowBeforeAllAssignments?: boolean;
    unanalyzable?: RuleUnanalyzable;
  }

  export type TestNoSkipModifier = "skip" | "skipIf" | "runIf" | "only" | "todo" | "fixme";

  export interface TestNoSkipsOptions {
    allow?: TestNoSkipModifier[];
    testModules?: string[];
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoConfigMirrorParser {
    module: string;
    name: string;
  }

  export interface TestNoConfigMirrorAssertionOptions {
    fixtureDirs?: string[];
    parsers?: TestNoConfigMirrorParser[];
  }

  export interface TestNoWallTimeAssertionsOptions {
    allowedPaths?: string[];
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TestNoTypedEnqueueFakeOptions {
    singleExportPattern?: string;
    moduleGlobs?: string[];
    unanalyzable?: RuleUnanalyzable;
  }

  export interface NoLocaleLessDateFormatOptions {
    methods?: string[];
    constructors?: string[];
    allowUndefinedLocale?: boolean;
    unknownValues?: "ignore" | "finding";
  }

  export type PostgresNoExecutorAdapterMode = "all" | "forwarding-only";

  export type PostgresUnanalyzable = "advise" | "report" | "ignore";

  export interface PostgresNonWritingCallee {
    module: string;
    export: string;
  }

  export interface PostgresNoExecutorAdapterOptions {
    importSpecifier?: string;
    executorNames?: string[];
    executorFactoryNames?: string[];
    executorTypeNames?: string[];
    trustedSqlTags?: { module: string; name: string }[];
    mode?: PostgresNoExecutorAdapterMode;
    nonWritingCallees?: PostgresNonWritingCallee[];
    unanalyzable?: PostgresUnanalyzable;
  }

  export interface PostgresPoolRequiresErrorListenerOptions {
    extraModules?: string[];
    unanalyzable?: PostgresUnanalyzable;
  }

  export interface TsNoConstAliasesOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TsNoExportRenamingOptions {
    allowDefaultReExports?: boolean;
    includePathPatterns?: string[];
    unanalyzable?: RuleUnanalyzable;
  }

  export interface ReactNoNullishReactNodeOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface ReactNoUsePromiseResolveOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface NextjsNoManualScriptTagsOptions {
    allowInlineScriptIds?: string[];
    allowInlineScriptIdPatterns?: string[];
    structuredDataComponent?: string;
    structuredDataMessage?: string;
    unanalyzable?: RuleUnanalyzable;
  }

  export interface ServerRequireNullableFetchWrapperOptions {
    includePathPatterns?: string[];
    excludePathPatterns?: string[];
    getterCalleePatterns: string[];
    requiredWrapperCallee: string;
    nullableReturnTypeNames?: string[];
    inferNullableFromTopLevelEntityPath?: boolean;
    topLevelEntityPathPatterns?: string[];
    unanalyzable?: RuleUnanalyzable;
  }
}

export = plugin;
