const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("pnpm release-age policy exports exclusive exact-name and pattern registry entries", () => {
  const declarations = readFileSync(
    join(packageRoot, "pnpm-release-age-policy-types.d.ts"),
    "utf8",
  );
  assert.match(
    declarations,
    /export type PnpmReleaseAgePermanentPackage =\s*\| \{ name: string; pattern\?: never; reason\?: string \}\s*\| \{ pattern: string; name\?: never; reason\?: string \};/,
  );
  assert.match(declarations, /export interface PnpmReleaseAgePolicyOptions \{/);
  assert.match(declarations, /permanentPackages\?: PnpmReleaseAgePermanentPackage\[\];/);
  assert.match(declarations, /temporaryGroups\?: PnpmReleaseAgeTemporaryGroup\[\];/);
  assert.match(declarations, /temporarySelectors\?: string\[\];/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/pnpm-release-age-policy-types";/,
  );
});
