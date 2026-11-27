const { assert, test, readFileSync, join, packageRoot } = require("./context");
const { withApi, envelope, fixtures } = require("./diagnostics-context");

test("PostgreSQL lock inventory DTOs and async facade are public", () => {
  const declarations = readFileSync(
    join(packageRoot, "postgres-lock-inventory-types.d.ts"),
    "utf8",
  );
  const typeExports = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  const api = readFileSync(join(packageRoot, "index-postgres-lock-inventory.d.ts"), "utf8");
  const diagnostics = readFileSync(join(packageRoot, "diagnostics-types.d.ts"), "utf8");

  for (const name of [
    "PostgresLockInventoryOptions",
    "PostgresLockInventoryRuleOptions",
    "LockInventorySite",
    "UnanalyzedLockSite",
    "LockInventoryBaselineSite",
    "LockInventoryBaselineUnanalyzed",
    "LockInventoryUnanalyzedDelta",
    "LockInventoryBaseline",
    "LockInventoryReport",
  ]) {
    assert.match(declarations, new RegExp(`export interface ${name} \\{`));
  }
  for (const field of [
    "root?: string;",
    "config?: string;",
    "writeBaseline?: boolean;",
    "baselineUnanalyzed?: boolean;",
    "sites: LockInventorySite[];",
    "counts: Record<string, number>;",
    "unanalyzed: UnanalyzedLockSite[];",
    "unanalyzedByReason: Record<string, number>;",
    "unanalyzedByDirectory: Record<string, number>;",
    "baselineFile: string;",
    "baselineUnanalyzed: boolean;",
    "added: LockInventorySite[];",
    "removed: LockInventoryBaselineSite[];",
    "unanalyzedAdded: LockInventoryUnanalyzedDelta[];",
    "unanalyzedRemoved: LockInventoryUnanalyzedDelta[];",
    "unanalyzed?: LockInventoryBaselineUnanalyzed[];",
    "baselineCount: number;",
    "currentCount: number;",
    "baselineUpdated: boolean;",
  ]) {
    assert.ok(declarations.includes(field), `missing public field ${field}`);
  }
  assert.match(typeExports, /export type \* from "\.\/postgres-lock-inventory-types"/);
  assert.match(
    api,
    /postgresLockInventory\(\s*options\?: WithInvocationOptions<PostgresLockInventoryOptions>,\s*\): Promise<WithInvocationResult<LockInventoryReport>>;/,
  );
  assert.match(index, /export \* from "\.\/index-postgres-lock-inventory";/);
  assert.match(
    readFileSync(join(packageRoot, "diagnostics-collector.d.ts"), "utf8"),
    /operation: "postgresLockInventory"[\s\S]*?Promise<DiagnosticsResult<Api.WithInvocationResult<Api.LockInventoryReport>>>/,
  );
  assert.match(diagnostics, /postgresLockInventory: typeof Api\.postgresLockInventory;/);
});

test("PostgreSQL lock inventory forwards options asynchronously and supports diagnostics", async () => {
  const report = {
    sites: [{ key: "stable-key", file: "src/query.ts", line: 4, category: "row-update" }],
    counts: { "row-update": 1 },
    unanalyzed: [
      { file: "src/orders/repo.ts", line: 9, reason: "executor SQL could not be resolved" },
    ],
    unanalyzedByReason: { "executor SQL could not be resolved": 1 },
    unanalyzedByDirectory: { "src/orders": 1 },
    baselineFile: ".no-mistakes/postgres-locks.json",
    baselineUnanalyzed: true,
    added: [],
    removed: [],
    unanalyzedAdded: [],
    unanalyzedRemoved: [],
    baselineUpdated: true,
  };
  const requests = [];

  await withApi(
    {
      postgresLockInventoryJson: async (json) => {
        const options = JSON.parse(json);
        requests.push(options);
        return options.__collectDiagnostics
          ? envelope(report, fixtures.diagnostics)
          : JSON.stringify(report);
      },
    },
    async (api) => {
      const options = {
        root: "/project",
        config: "/project/.no-mistakes.yml",
        writeBaseline: true,
      };
      const pending = api.postgresLockInventory(options);
      assert.equal(typeof pending.then, "function");
      assert.deepEqual(await pending, report);

      const collected = await api.collectDiagnostics("postgresLockInventory", options);
      assert.deepEqual(collected, { result: report, diagnostics: fixtures.diagnostics });
    },
  );

  assert.deepEqual(requests, [
    {
      root: "/project",
      config: "/project/.no-mistakes.yml",
      writeBaseline: true,
    },
    {
      root: "/project",
      config: "/project/.no-mistakes.yml",
      writeBaseline: true,
      __collectDiagnostics: true,
    },
  ]);
});
