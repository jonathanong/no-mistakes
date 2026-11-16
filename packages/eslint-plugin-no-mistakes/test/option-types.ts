import plugin, {
  type PostgresNoExecutorAdapterMode,
  type PostgresNoExecutorAdapterOptions,
} from "../src/index.js";
import pluginRequire = require("../src/index.js");

export const mode: PostgresNoExecutorAdapterMode = "forwarding-only";
export const options: PostgresNoExecutorAdapterOptions = {
  importSpecifier: "@example/db",
  executorTypeNames: ["QueryExecutor"],
  mode,
};
export const rule = plugin.rules?.["postgres-no-executor-adapter"];
export const commonJsRule = pluginRequire.rules?.["postgres-no-executor-adapter"];

// @ts-expect-error Unknown mode must fail at the declaration boundary.
export const invalid: PostgresNoExecutorAdapterMode = "quiet";
