import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, lint, messages } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const mockHelpers = require("../src/rules/module-mock-helpers");
const preserveAliases = require("../src/rules/module-mock-preserve-aliases");

function fixture(name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/upstreamed-generic/fixture", name),
    "utf8",
  );
}

const integrationPreserveFixtureRoot = resolve(
  __dirname,
  "../../../fixtures/eslint-plugin/module-mock-integration-preserve",
);

function integrationPreserveFixture(name) {
  return readFileSync(resolve(integrationPreserveFixtureRoot, name), "utf8");
}

function integrationPreserveRules() {
  const options = {
    integrationExports: {
      specifiers: ["@app/**"],
      sourcePathTemplates: [resolve(integrationPreserveFixtureRoot, "integration.ts")],
    },
    internalSpecifiers: ["@app/**"],
  };
  return {
    "no-mistakes/module-mock-boundary": ["error", options],
    "no-mistakes/module-mock-preserve-exports": ["error", options],
  };
}

const integrationBarrelFixtureRoot = resolve(
  __dirname,
  "../../../fixtures/eslint-plugin/module-mock-integration-barrel",
);

function barrelRules(sourceFile = "index.ts", overrides = {}) {
  const options = {
    integrationExports: {
      specifiers: ["@app/**"],
      sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, sourceFile)],
      ...overrides,
    },
    internalSpecifiers: ["@app/**"],
  };
  return {
    "no-mistakes/module-mock-boundary": ["error", options],
    "no-mistakes/module-mock-preserve-exports": ["error", options],
  };
}

describe("module mock integration export preservation", () => {
  it("allows proven real-module spreads through both rules", () => {
    assert.deepEqual(
      lint(
        integrationPreserveFixture("valid.test.ts"),
        integrationPreserveRules(),
        "fixtures/module-mock-integration-preserve/valid.test.ts",
      ),
      [],
    );
  });

  it("fails closed without granting explicit overrides permission", () => {
    assert.deepEqual(
      lint(
        integrationPreserveFixture("invalid.test.ts"),
        integrationPreserveRules(),
        "fixtures/module-mock-integration-preserve/invalid.test.ts",
      )
        .map((message) => message.messageId)
        .sort(),
      [
        "boundary",
        "boundary",
        "boundary",
        "boundary",
        "boundary",
        "boundary",
        "boundary",
        "boundary",
        "preserve",
        "preserve",
        "preserve",
        "preserve",
        "preserve",
        "preserve",
      ],
    );
  });
});

describe("module-mock-boundary barrel re-exports", () => {
  it("allows a spread-preserving mock of a barrel entrypoint tagged on a re-exported leaf", () => {
    assert.deepEqual(
      lint(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", async (importOriginal) => ({
            ...(await importOriginal()),
            taggedProviderCall: vi.fn(),
          }));
        `,
        barrelRules(),
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("allows a plain-object mock of a barrel entrypoint tagged on a re-exported leaf", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "index.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("follows a multi-level barrel chain including directory-index resolution", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ nestedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "index.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("terminates and allows a tagged export reached through a re-export cycle", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ cycledProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "cycle-a.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("still blocks untagged exports reached through the barrel (monotonic, never over-allows)", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ untaggedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "index.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("leaves bare-specifier re-exports unresolved", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ anything: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "bare-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("does not throw when a local re-export target is missing on disk", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ anything: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "missing-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("honors a reexportExtensions override that excludes the leaf's actual extension", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "index.ts")],
            reexportExtensions: [".mjs"],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("ignores re-export syntax inside comments while still following live re-exports", () => {
    const options = {
      integrationExports: {
        specifiers: ["@app/**"],
        sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "commented-barrel.ts")],
      },
      internalSpecifiers: ["@app/**"],
    };
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        options,
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ nestedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        options,
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("does not propagate a tagged default export through export-star barrels", () => {
    const mockDefault = `
      import { vi } from "vitest";
      vi.mock("@app/aws", () => ({ default: vi.fn() }));
    `;
    // Reached only through `export *`: ES modules never re-export a target's
    // default binding this way, so the barrel must not expose it either.
    assert.deepEqual(
      messages(
        mockDefault,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "default-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
    // Mocked as the direct root specifier (no barrel involved): the tagged default
    // export is collected normally.
    assert.deepEqual(
      messages(
        mockDefault,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "default-leaf.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("resolves a re-export specifier carrying a compiled .js extension to its .ts source", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "esm-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("resolves an extensionless re-export specifier to a .tsx leaf by default", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedButtonProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "tsx-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("does not propagate a named-export-list default alias through export-star barrels", () => {
    const mockDefault = `
      import { vi } from "vitest";
      vi.mock("@app/aws", () => ({ default: vi.fn() }));
    `;
    // Reached only through `export *`: the tag sits on `export { x as default }`
    // rather than an `export default` declaration, but the same ES module rule
    // applies — star re-exports never include the target's default binding.
    assert.deepEqual(
      messages(
        mockDefault,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "named-default-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
    // Mocked as the direct root specifier (no barrel involved): the tagged
    // `export { x as default }` alias is collected normally.
    assert.deepEqual(
      messages(
        mockDefault,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "named-default-leaf.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("resolves a .js specifier to its .ts sibling, not an ambiguous .mts sibling", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedAmbiguousProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "ambiguous-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("resolves a .jsx specifier to its .tsx source", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedButtonProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "jsx-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("does not fall back to generic extension/directory probing for a compiled-extension specifier", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/aws", () => ({ taggedFallbackProviderCall: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifiers: ["@app/**"],
            sourcePathTemplates: [resolve(integrationBarrelFixtureRoot, "fallback-barrel.ts")],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });
});

describe("module-mock-preserve-exports", () => {
  it("requires internal mock factories to preserve real exports", () => {
    assert.deepEqual(
      messages(
        fixture("module-mock-preserve-exports.test.ts"),
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve", "preserve"],
    );
  });

  it("supports baselines and path filters", () => {
    const code = fixture("module-mock-preserve-exports.test.ts");
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        {
          baseline: [["tests/module-mock-preserve-exports.test.ts", "./invalid-partial"]],
          includePathPatterns: ["tests/**"],
          internalSpecifiers: ["./**"],
        },
        "tests/module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve"],
    );
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { includePathPatterns: ["backend/**"], internalSpecifiers: ["./**"] },
        "tests/module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("covers dynamic, external, no-factory, and non-function preserve branches", () => {
    const code = `
      import { vi } from "vitest";
      const name = "client";
      vi.mock(\`./\${name}\`, () => ({ run: vi.fn() }));
      vi.mock("external", () => ({ run: vi.fn() }));
      vi.mock("./missing-factory");
      vi.mock("./options", { spy: true });
      vi.mock("./non-function", factory);
      vi.mock("./bad-resolver", async () => ({
        ...(await vi.requireActual("./bad-resolver")),
        run: vi.fn(),
      }));
      vi.mock("./bare-import-original", async () => ({
        ...(await importOriginal()),
        run: vi.fn(),
      }));
      client.mock("./not-framework", () => ({ run: vi.fn() }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**", "/^@app\\/.+/"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve", "preserve"],
    );
  });

  it("checks ESM and namespace-imported framework mocks", () => {
    const code = `
      import * as vitest from "vitest";
      import { jest } from "@jest/globals";
      const mockModule = jest.unstable_mockModule;
      vitest.vi.unstable_mockModule("./esm", () => ({ run: vitest.vi.fn() }));
      mockModule("./esm-alias", () => ({ run: jest.fn() }));
      vitest.vi.mock("./namespace-bad", () => ({ run: vitest.vi.fn() }));
      vitest.vi.mock("./namespace-good", async () => ({
        ...(await vitest.vi.importActual("./namespace-good")),
        run: vitest.vi.fn(),
      }));
      vitest.vi.mock("./try-partial", async () => {
        try {
          return { run: vitest.vi.fn() };
        } catch {
          return {
            ...(await vitest.vi.importActual("./try-partial")),
            run: vitest.vi.fn(),
          };
        }
      });
      jest.mock("./awaited-jest", async () => ({
        ...(await jest.requireActual("./awaited-jest")),
        run: jest.fn(),
      }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve", "preserve", "preserve"],
    );
  });
});

describe("module-mock-boundary", () => {
  it("bans configured internal module mocks and allows integration exports", () => {
    assert.deepEqual(
      messages(
        fixture("module-mock-boundary.test.ts"),
        "module-mock-boundary",
        {
          baseline: [["module-mock-boundary.test.ts", "@app/baselined", 1]],
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary", "boundary", "boundary", "boundary", "boundary"],
    );
  });

  it("reports stale counted baselines", () => {
    assert.deepEqual(
      messages(
        "import { vi } from 'vitest';\n",
        "module-mock-boundary",
        {
          baseline: [["module-mock-boundary.test.ts", "@app/old", 1]],
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["stale"],
    );
  });

  it("supports opt-outs and opaque integration factories", () => {
    const code = `
      import { vi } from "vitest";
      const name = "service";
      vi.mock(\`@app/\${name}\`, () => ({ run: vi.fn() }));
      vi.mock("@app/opaque", () => factory());
      vi.mock("@app/missing-source", () => ({ allowed: vi.fn() }));
      vi.mock("@app/computed", () => ({ [name]: vi.fn() }));
      other.mock("@app/not-framework", () => ({ run: vi.fn() }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-boundary",
        {
          excludePathPatterns: ["**/excluded.test.ts"],
          internalSpecifiers: ["@app/**"],
        },
        "tests/excluded.test.ts",
      ),
      [],
    );
    assert.deepEqual(
      messages(
        code,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/opaque", "@app/missing-source", "@app/computed"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
          requireLiteralSpecifiers: false,
        },
        "tests/included.test.ts",
      ),
      ["boundary", "boundary"],
    );
  });

  it("fails closed for opaque integration factory return paths", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/integration", () => {
            if (flag) return makePartialMock();
            return { allowed: vi.fn() };
          });
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("allows named and default marker exports through integration config", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/integration", () => ({ namedAllowed: vi.fn() }));
          vi.mock("@app/integration", function () {
            return { namedAllowed: vi.fn() };
          });
          vi.mock("@app/integration", () => ({ default: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("reports when integration source templates do not resolve", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/missing-source", () => ({ allowed: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/missing-source"],
            sourcePathTemplates: ["missing/module/{specifierSuffix}{extension}"],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("tracks assignment aliases and ignores non-mock imports", () => {
    const code = `
      import { fn } from "vitest";
      import { mock as directMock, vi } from "vitest";
      let mockLater;
      mockLater = vi.mock;
      mockLater += vi.doMock;
      const mockCall = vi.mock;
      directMock("@app/imported", () => ({ run: vi.fn() }));
      mockCall.call(undefined, "@app/called", () => ({ run: vi.fn() }));
      mockLater("@app/later", () => ({ run: vi.fn() }));
      mockLater.apply(undefined, ["@app/applied", () => ({ run: vi.fn() })]);
      mockLater.apply(undefined, args);
      fn("@app/not-mock");
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary", "boundary", "boundary", "boundary", "boundary"],
    );
  });

  it("resolves namespace-imported framework mocks", () => {
    assert.deepEqual(
      messages(
        `
          import * as vitest from "vitest";
          vitest.vi.mock("@app/service", () => ({ run: vitest.vi.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("does not treat non-mock framework imports as namespaces", () => {
    assert.deepEqual(
      messages(
        `
          import { expect } from "vitest";
          expect.mock("@app/service", () => ({ run: expect.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("inspects direct framework mock call and apply invocations", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock.call(undefined, "@app/called", () => ({ run: vi.fn() }));
          vi.mock.apply(undefined, ["@app/applied", () => ({ run: vi.fn() })]);
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary", "boundary"],
    );
  });
});

describe("module mock helpers", () => {
  it("covers glob, literal, import, and pattern helpers", () => {
    assert.equal(mockHelpers.stringMatches("@app/a/b", ["@app/**"]), true);
    assert.equal(mockHelpers.stringMatches("@app/a", ["/^@app\\/[a-z]$/"]), true);
    assert.equal(mockHelpers.stringMatches("@app/a1", ["@app/?"]), false);
    assert.equal(mockHelpers.propertyName(), null);
    assert.equal(
      mockHelpers.literalString({
        expressions: [],
        quasis: [{ value: { raw: "client" } }],
        type: "TemplateLiteral",
      }),
      "client",
    );
    assert.equal(mockHelpers.literalString(), null);
    assert.equal(
      mockHelpers.repoRelativeFilename(resolve(process.cwd(), "packages/app/src/file.ts")),
      "packages/app/src/file.ts",
    );
    assert.equal(
      mockHelpers.isFrameworkBinding(
        { type: "MemberExpression" },
        { sourceCode: { getScope: () => null } },
      ),
      false,
    );
    assert.deepEqual(
      mockHelpers.moduleMockSpecifierArgument({
        source: { type: "Literal", value: "./client" },
        type: "ImportExpression",
      }),
      { dynamic: false, specifier: "./client" },
    );
    assert.deepEqual(
      mockHelpers.moduleMockSpecifierArgument({
        arguments: [{ type: "Identifier", name: "name" }],
        callee: { type: "Import" },
        type: "CallExpression",
      }),
      { dynamic: true },
    );
    assert.deepEqual(
      mockHelpers.moduleMockSpecifierArgument({
        source: { type: "Identifier", name: "specifier" },
        type: "ImportExpression",
      }),
      { dynamic: true },
    );
    assert.equal(mockHelpers.importSpecifierName({}), null);
    assert.deepEqual([...mockHelpers.collectPatternNames()], []);
    assert.deepEqual(
      [
        ...mockHelpers.collectPatternNames({
          elements: [{ name: "first", type: "Identifier" }],
          type: "ArrayPattern",
        }),
      ],
      ["first"],
    );
    assert.deepEqual(
      [
        ...mockHelpers.collectPatternNames({
          argument: { name: "rest", type: "Identifier" },
          type: "RestElement",
        }),
      ],
      ["rest"],
    );
    assert.equal(mockHelpers.stringMatches("src/app.ts", ["src/*.ts"]), true);
    assert.equal(mockHelpers.stringMatches("root.test.ts", ["**/*.test.ts"]), true);
    assert.equal(mockHelpers.stringMatches("src/root.test.ts", ["**/*.test.ts"]), true);
    assert.equal(mockHelpers.stringMatches("src/app.ts", []), false);
    assert.equal(mockHelpers.stringMatches("@app/a", ["/(/"]), false);
    assert.equal(
      mockHelpers.pathAllowed("tests/file.test.ts", {
        excludePathPatterns: ["tests/**"],
        includePathPatterns: ["**/*.test.ts"],
      }),
      false,
    );
    assert.equal(mockHelpers.isInternalSpecifier("./client", {}), true);
    assert.equal(
      mockHelpers.expressionName({
        computed: false,
        object: { name: "vitest", type: "Identifier" },
        property: { name: "vi", type: "Identifier" },
        type: "MemberExpression",
      }),
      "vitest.vi",
    );
    assert.equal(
      mockHelpers.expressionName({
        computed: true,
        object: { name: "vitest", type: "Identifier" },
        property: { name: "vi", type: "Identifier" },
        type: "MemberExpression",
      }),
      null,
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "vi", type: "Identifier" },
        { sourceCode: { getScope: () => null } },
      ),
      "vitest",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "j", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  defs: [
                    {
                      node: {
                        imported: { name: "jest", type: "Identifier" },
                        type: "ImportSpecifier",
                      },
                      parent: { source: { value: "@jest/globals" } },
                      type: "ImportBinding",
                    },
                  ],
                  name: "j",
                },
              ],
            }),
          },
        },
      ),
      "@jest/globals",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "jest", type: "Identifier" },
        { sourceCode: { getScope: () => null } },
      ),
      "@jest/globals",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "vi", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [{ defs: [], name: "vi" }],
            }),
          },
        },
      ),
      "vitest",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "jest", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [{ defs: [], name: "jest" }],
            }),
          },
        },
      ),
      "@jest/globals",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "client", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [{ defs: [{ node: {}, type: "Variable" }], name: "client" }],
            }),
          },
        },
      ),
      null,
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "mocked", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  defs: [
                    {
                      node: {
                        init: {
                          arguments: [{ value: "vitest" }],
                          callee: { name: "require", type: "Identifier" },
                          type: "CallExpression",
                        },
                      },
                      type: "Variable",
                    },
                  ],
                  name: "mocked",
                },
              ],
            }),
          },
        },
      ),
      "vitest",
    );
    assert.equal(
      mockHelpers.frameworkBindingModule(
        { name: "computed", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: null,
              variables: [
                {
                  defs: [
                    {
                      node: {
                        init: {
                          computed: true,
                          object: {
                            arguments: [{ value: "vitest" }],
                            callee: { name: "require", type: "Identifier" },
                            type: "CallExpression",
                          },
                          property: { type: "Literal", value: "vi" },
                          type: "MemberExpression",
                        },
                      },
                      type: "Variable",
                    },
                  ],
                  name: "computed",
                },
              ],
            }),
          },
        },
      ),
      "vitest",
    );
    assert.equal(
      preserveAliases.resolveVariable(
        { name: "missing", type: "Identifier" },
        {
          sourceCode: {
            getScope: () => ({
              upper: {
                upper: null,
                variables: [],
              },
              variables: [],
            }),
          },
        },
      ),
      null,
    );
    const aliasTracker = preserveAliases.createPreserveMockAliases({
      sourceCode: {
        getScope: () => ({
          upper: {
            upper: null,
            variables: [],
          },
          variables: [],
        }),
      },
    });
    aliasTracker.declare(
      {
        elements: [{ name: "arrayAlias", type: "Identifier" }],
        type: "ArrayPattern",
      },
      {
        computed: false,
        object: { name: "vi", type: "Identifier" },
        property: { name: "mock", type: "Identifier" },
        type: "MemberExpression",
      },
    );
    assert.deepEqual(aliasTracker.get({ name: "arrayAlias", type: "Identifier" }), {
      framework: "vitest",
      method: "mock",
      namespace: "vi",
    });
    assert.equal(
      mockHelpers.importSpecifierName({ imported: { type: "Literal", value: "mock" } }),
      "mock",
    );
    assert.deepEqual(
      mockHelpers.moduleMockSpecifierArgument({
        arguments: [{ type: "Literal", value: "./client" }],
        callee: { type: "Import" },
        type: "CallExpression",
      }),
      { dynamic: false, specifier: "./client" },
    );
    assert.deepEqual(
      [
        ...mockHelpers.collectPatternNames({
          left: { name: "assigned", type: "Identifier" },
          type: "AssignmentPattern",
        }),
      ],
      ["assigned"],
    );
    assert.deepEqual(
      [
        ...mockHelpers.collectPatternNames({
          properties: [
            {
              argument: { name: "restObject", type: "Identifier" },
              type: "RestElement",
            },
          ],
          type: "ObjectPattern",
        }),
      ],
      ["restObject"],
    );
  });

  it("recognizes require-bound framework bindings", () => {
    const code = `
      const vi = require("vitest").vi;
      vi.mock("@app/service", () => ({ run: vi.fn() }));
    `;
    assert.equal(
      lint(
        code,
        { "no-mistakes/module-mock-boundary": ["error", { internalSpecifiers: ["@app/**"] }] },
        "module-mock-boundary.test.cjs",
      ).length,
      1,
    );
  });

  it("covers assignment-pattern importOriginal factories", () => {
    const code = `
      import { vi } from "vitest";
      vi.mock("./client", async (importOriginal = fallback) => ({
        ...(await importOriginal()),
        run: vi.fn(),
      }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("allows aliased Vitest importOriginal factories", () => {
    const code = `
      import { vi as v } from "vitest";
      v.mock("./client", async (importOriginal) => ({
        ...(await importOriginal()),
        run: v.fn(),
      }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("allows aliased Vitest namespace importActual calls", () => {
    const code = `
      import { vi as v } from "vitest";
      v.mock("./client", async () => ({
        ...(await v.importActual("./client")),
        run: v.fn(),
      }));
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("allows TypeScript-cast spread arguments and branch-local actual aliases", () => {
    const code = `
      import { vi } from "vitest";
      vi.mock("./typed", async () => ({
        ...((await vi.importActual("./typed")) as typeof import("./typed")),
        run: vi.fn(),
      }));
      vi.mock("./conditional", async () => {
        if (enabled) {
          const actual = await vi.importActual("./conditional");
          return { ...actual, run: vi.fn() };
        }
        const fallbackActual = await vi.importActual("./conditional");
        return { ...fallbackActual, run: vi.fn() };
      });
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("checks aliased preserve mocks and shadowed actual-module aliases", () => {
    const code = `
      import { vi } from "vitest";
      const mock = vi.mock;
      mock("./aliased", () => ({ run: vi.fn() }));
      let doMock;
      doMock = vi.doMock;
      doMock("./assigned", () => ({ run: vi.fn() }));
      function run(mock) {
        mock("./shadowed-alias", () => ({ run: vi.fn() }));
      }
      vi.mock("./shadowed", async () => {
        const actual = await vi.importActual("./shadowed");
        if (enabled) {
          const actual = { run: vi.fn() };
          return { ...actual };
        }
        return { ...actual, run: vi.fn() };
      });
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve", "preserve"],
    );
  });

  it("checks destructured preserve aliases and aliased Jest namespaces", () => {
    assert.deepEqual(
      messages(
        `
          import { jest as j } from "@jest/globals";
          import { vi } from "vitest";
          const { mock } = vi;
          mock("./aliased", () => ({ run: vi.fn() }));
          const jestMock = j.mock;
          jestMock("./client", () => ({ ...j.requireActual("./client"), run: j.fn() }));
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve"],
    );
  });

  it("checks directly imported preserve mocks and alias call/apply invocations", () => {
    assert.deepEqual(
      messages(
        `
          import { mock } from "vitest";
          import { vi } from "vitest";
          const alias = vi.mock;
          mock("./direct", () => ({ run: vi.fn() }));
          alias.call(undefined, "./called", () => ({ run: vi.fn() }));
          alias.apply(undefined, ["./applied", () => ({ run: vi.fn() })]);
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve", "preserve", "preserve"],
    );
  });

  it("rejects opaque return paths in preserve mock factories", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("./client", async () => {
            if (flag) return makePartialMock();
            return { ...(await vi.importActual("./client")), run: vi.fn() };
          });
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve"],
    );
  });

  it("tracks defaulted destructured mock aliases with real scope resolution", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          const { mock: localMock = vi.mock } = vi;
          localMock("./client", () => ({ run: vi.fn() }));
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve"],
    );
  });

  it("ignores dynamic computed mock members and shadowed preserve alias bindings", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          const mockName = "mock";
          vi[mockName]("@app/service", () => ({ run: vi.fn() }));
          vi["mock"]("@app/static", () => ({ run: vi.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          const mock = vi.mock;
          {
            const mock = fakeHarness;
            mock("./client", () => ({ run: vi.fn() }));
          }
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("ignores nested helper returns inside preserving mock factories", () => {
    const code = `
      import { vi } from "vitest";
      vi.mock("./client", async () => {
        function helperFactory() {
          return { run: vi.fn() };
        }
        const helper = () => ({ run: vi.fn() });
        return { ...(await vi.importActual("./client")), run: vi.fn(), helper };
      });
    `;
    assert.deepEqual(
      messages(
        code,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      [],
    );
  });

  it("checks nested and TypeScript-wrapped mock factory returns", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("./conditional", async () => {
            if (enabled) {
              return { run: vi.fn() };
            }
            return { ...(await vi.importActual("./conditional")), run: vi.fn() };
          });
          vi.mock("./satisfies", async () =>
            ({ ...(await vi.importActual("./satisfies")), run: vi.fn() } satisfies Partial<
              typeof import("./satisfies")
            >)
          );
        `,
        "module-mock-preserve-exports",
        { internalSpecifiers: ["./**"] },
        "module-mock-preserve-exports.test.ts",
      ),
      ["preserve"],
    );
  });

  it("matches root files for globstar path filters", () => {
    assert.deepEqual(
      messages(
        `import { vi } from "vitest"; vi.mock("@app/root", () => ({ run: vi.fn() }));`,
        "module-mock-boundary",
        { includePathPatterns: ["**/*.test.ts"], internalSpecifiers: ["@app/**"] },
        "root.test.ts",
      ),
      ["boundary"],
    );
  });

  it("reports stale baselines for absolute filenames", () => {
    assert.deepEqual(
      messages(
        "import { vi } from 'vitest';\n",
        "module-mock-boundary",
        {
          baseline: [["tests/module-mock-boundary.test.ts", "@app/old", 1]],
          internalSpecifiers: ["@app/**"],
        },
        resolve(process.cwd(), "tests/module-mock-boundary.test.ts"),
      ),
      ["stale"],
    );
  });

  it("tracks destructured framework mock aliases", () => {
    assert.deepEqual(
      messages(
        `
          import { mock as externalMock } from "external";
          import { vi } from "vitest";
          const { mock, notMock, ...rest } = vi;
          mock("@app/service", () => ({ run: vi.fn() }));
          notMock("@app/ignored", () => ({ run: vi.fn() }));
          externalMock("@app/external", () => ({ run: vi.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("reports importMock boundary calls", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.importMock("@app/service");
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("does not treat local vi variables as framework globals", () => {
    assert.deepEqual(
      messages(
        `
          function run(vi) {
            vi.mock("@app/service", () => ({ run: vi.fn() }));
          }
          const jest = fakeHarness;
          jest.mock("@app/other", () => ({ run: jest.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("does not treat shadowed boundary aliases as framework mocks", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          const mock = vi.mock;
          {
            const mock = fakeHarness;
            mock("@app/service", () => ({ run: vi.fn() }));
          }
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("ignores shadowed mock alias names", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          const mock = vi.mock;
          function callLocal(mock) {
            mock("@app/local", () => ({ run: vi.fn() }));
          }
          mock("@app/service", () => ({ run: vi.fn() }));
        `,
        "module-mock-boundary",
        { internalSpecifiers: ["@app/**"] },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });

  it("checks all integration return paths and rejects opaque spreads", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/integration", () => {
            if (flag) {
              return { blocked: vi.fn() };
            }
            return { allowed: vi.fn() };
          });
          vi.mock("@app/integration", () => ({ ...partialMock, allowed: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary", "boundary"],
    );
  });

  it("unwraps TypeScript integration mock objects", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/integration", () => ({ allowed: vi.fn() } satisfies Partial<unknown>));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      [],
    );
  });

  it("treats invalid integration marker regex config as not allowing the mock", () => {
    assert.deepEqual(
      messages(
        `
          import { vi } from "vitest";
          vi.mock("@app/integration", () => ({ allowed: vi.fn() }));
        `,
        "module-mock-boundary",
        {
          integrationExports: {
            markerRegex: "(",
            specifierPrefix: "@app/",
            specifiers: ["@app/integration"],
            sourcePathTemplates: [
              resolve(
                __dirname,
                "../../../test-cases/eslint-plugin/upstreamed-generic/fixture/module-mock-boundary-integration.ts",
              ),
            ],
          },
          internalSpecifiers: ["@app/**"],
        },
        "module-mock-boundary.test.ts",
      ),
      ["boundary"],
    );
  });
});
