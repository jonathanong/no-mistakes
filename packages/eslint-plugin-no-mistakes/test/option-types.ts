import plugin, {
  type NoTopLevelAwaitOptions,
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

export const awaitOptions: NoTopLevelAwaitOptions = {
  allowInImportMetaMain: false,
  message: "Initialize explicitly",
};
// @ts-expect-error Guard policy must be a boolean.
export const invalidAwaitOptions: NoTopLevelAwaitOptions = { allowInImportMetaMain: "yes" };
