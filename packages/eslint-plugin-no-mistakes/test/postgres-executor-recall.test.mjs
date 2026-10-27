import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const DB = { importSpecifier: "@example/db" };

function recall(name, options = DB) {
  return {
    code: fixture(`postgres-executor-recall/${name}.ts`),
    filename: "fixture.ts",
    options: [options],
    languageOptions: { parser: tsParser },
  };
}

const manual = {
  valid: [
    // manual-transaction follows static aliases and one forwarder.
    // Zero findings.
    recall("manual", { executorNames: [] }),
    // manual-transaction preserves suppression at the forwarded call.
    // Zero findings.
    recall("suppressed-manual-transaction"),
    recall("file-manual-transaction"),
    // does not follow unknown, mutable, shadowed or second-hop calls.
    // Zero findings.
    recall("silent"),
  ],
  invalid: [
    // manual-transaction follows static aliases and one forwarder.
    // Nine manualTransaction findings.
    recall("manual"),
    // matches configured call/apply exports before forwarding.
    // Three manualTransaction findings.
    recall("configured-methods", { ...DB, executorNames: ["call", "apply"] }),
    // preserves scoped factory/type executors and their lexical boundaries.
    // Two manualTransaction findings.
    recall("scoped", {
      ...DB,
      executorFactoryNames: ["makeExecutor"],
      executorTypeNames: ["Executor"],
    }),
    // follows namespace aliases, scoped query destructures, and awaited forwarders.
    // Nine manualTransaction findings.
    recall("more-static"),
  ],
};

const registry = {
  valid: [
    // nonliteral-sql-registry-key follows static aliases and one forwarder.
    // Zero findings.
    recall("registry", { executorNames: [] }),
    // nonliteral-sql-registry-key preserves suppression at the forwarded call.
    // Zero findings.
    recall("suppressed-nonliteral-sql-registry-key"),
    recall("file-nonliteral-sql-registry-key"),
    // does not follow unknown, mutable, shadowed or second-hop calls.
    // Zero findings.
    recall("registry-silent"),
  ],
  invalid: [
    // nonliteral-sql-registry-key follows static aliases and one forwarder.
    // Nine dynamicKey findings.
    recall("registry"),
  ],
};

const fanout = {
  valid: [
    // unbounded-query-fanout follows static aliases and one forwarder.
    // Zero findings.
    recall("fanout", { executorNames: [] }),
    // unbounded-query-fanout preserves suppression at the forwarded call.
    // Zero findings.
    recall("suppressed-unbounded-query-fanout"),
    recall("file-unbounded-query-fanout"),
    // does not follow unknown, mutable, shadowed or second-hop calls.
    // Zero findings.
    recall("fanout-silent"),
    recall("silent"),
  ],
  invalid: [
    // unbounded-query-fanout follows static aliases and one forwarder.
    // 12 unboundedFanout findings.
    recall("fanout"),
    // matches configured call/apply exports before forwarding.
    // Two unboundedFanout findings.
    recall("configured-methods", { ...DB, executorNames: ["call", "apply"] }),
  ],
};

it("snapshots postgres executor recall diagnostics", async () => {
  await expect(
    snapshotDocument({
      "postgres-no-manual-transaction": snapshotSuite(
        "postgres-no-manual-transaction",
        languageOptions,
        manual,
      ),
      "postgres-no-nonliteral-sql-registry-key": snapshotSuite(
        "postgres-no-nonliteral-sql-registry-key",
        languageOptions,
        registry,
      ),
      "postgres-no-unbounded-query-fanout": snapshotSuite(
        "postgres-no-unbounded-query-fanout",
        languageOptions,
        fanout,
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-executor-recall.snap");
}, 30_000);
