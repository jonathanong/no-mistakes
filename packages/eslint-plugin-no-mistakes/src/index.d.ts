import type { ESLint } from "eslint";

declare const plugin: ESLint.Plugin;
declare namespace plugin {
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
