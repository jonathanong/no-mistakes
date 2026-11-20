import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const filename = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/executor-target/noncallable-client.ts",
);
const code = fixture("executor-target/noncallable-client.ts");
const option = {
  importSpecifier: "@example/db",
  executorTypeNames: ["PoolClient", "QueryWithClient", "SessionHandle"],
};

it("snapshots callable and non-callable passed executor messages", async () => {
  await expect(
    snapshotDocument({
      "postgres-no-executor-adapter": snapshotSuite(
        "postgres-no-executor-adapter",
        { ecmaVersion: 2024, sourceType: "module", globals: {} },
        {
          invalid: [
            // A callable executor keeps the call form. A pool client names query.
            // A type with neither names the executor and omits the call.
            {
              code,
              filename,
              options: [option],
              languageOptions: {
                parser: tsParser,
                parserOptions: {
                  project: resolve(dirname(filename), "tsconfig.json"),
                  disallowAutomaticSingleRunInference: true,
                },
              },
            },
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-noncallable-executor.snap");
}, 30_000);

it("keeps the call form when no TypeScript program is available", async () => {
  const name = "untyped-call-form.ts";
  const file = resolve(
    __dirname,
    "../../../test-cases/eslint-snippets/fixture/executor-target",
    name,
  );
  await expect(
    snapshotDocument({
      "postgres-no-executor-adapter": snapshotSuite(
        "postgres-no-executor-adapter",
        { ecmaVersion: 2024, sourceType: "module", globals: {} },
        {
          invalid: [
            {
              code: fixture(`executor-target/${name}`),
              filename: file,
              options: [option],
              languageOptions: { parser: tsParser },
            },
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("snapshots/postgres-untyped-call-form.snap");
});
