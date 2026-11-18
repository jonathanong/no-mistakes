import type { ESLint } from "eslint";

declare const plugin: ESLint.Plugin;
declare namespace plugin {
  export interface NoTopLevelAwaitOptions {
    allowInImportMetaMain?: boolean;
    message?: string;
  }

  export type PostgresNoExecutorAdapterMode = "all" | "forwarding-only";

  export interface PostgresNoExecutorAdapterOptions {
    importSpecifier?: string;
    executorNames?: string[];
    executorFactoryNames?: string[];
    executorTypeNames?: string[];
    trustedSqlTags?: { module: string; name: string }[];
    mode?: PostgresNoExecutorAdapterMode;
  }
}

export = plugin;
