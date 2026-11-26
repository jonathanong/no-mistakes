const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("lockfile diff manager union includes ecosystem lockfiles", () => {
  assert.match(
    readFileSync(join(packageRoot, "test-types.d.ts"), "utf8"),
    /manager:\s*\n\s*\| "npm"\s*\n\s*\| "pnpm"\s*\n\s*\| "yarn"\s*\n\s*\| "bun"\s*\n\s*\| "poetry"\s*\n\s*\| "uv"\s*\n\s*\| "pipfile"\s*\n\s*\| "go"\s*\n\s*\| "cargo"\s*\n\s*\| "gemfile"\s*\n\s*\| "composer";/,
  );
});
