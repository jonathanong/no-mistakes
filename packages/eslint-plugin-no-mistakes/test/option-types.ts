import plugin, {
  type NoTopLevelAwaitOptions,
  type NoLocaleLessDateFormatOptions,
  type PostgresNoExecutorAdapterMode,
  type PostgresNonWritingCallee,
  type PostgresNoExecutorAdapterOptions,
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
  allowUndefinedLocale: true,
};
export const defaultDateFormatOptions: pluginRequire.NoLocaleLessDateFormatOptions = {};

// @ts-expect-error Methods must be an array of strings.
export const invalidMethods: NoLocaleLessDateFormatOptions = { methods: [1] };
export const invalidAllowUndefinedLocale: NoLocaleLessDateFormatOptions = {
  // @ts-expect-error The undefined-locale option must be boolean.
  allowUndefinedLocale: "true",
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
