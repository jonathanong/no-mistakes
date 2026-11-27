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

  export interface TestNoWallTimeAssertionsOptions {
    allowedPaths?: string[];
  }

  export interface TestNoTypedEnqueueFakeOptions {
    singleExportPattern?: string;
    moduleGlobs?: string[];
  }

  export interface NoLocaleLessDateFormatOptions {
    methods?: string[];
    constructors?: string[];
    allowUndefinedLocale?: boolean;
    unknownValues?: "ignore" | "finding";
  }

  export type PostgresNoExecutorAdapterMode = "all" | "forwarding-only";

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
  }

  export interface PostgresPoolRequiresErrorListenerOptions {
    extraModules?: string[];
  }

  export type RuleUnanalyzable = "finding" | "ignore";

  export interface TsNoConstAliasesOptions {
    unanalyzable?: RuleUnanalyzable;
  }

  export interface TsNoExportRenamingOptions {
    allowDefaultReExports?: boolean;
    includePathPatterns?: string[];
    unanalyzable?: RuleUnanalyzable;
  }
}

export = plugin;
