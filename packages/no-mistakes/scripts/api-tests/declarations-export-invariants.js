const { readdirSync } = require("node:fs");
const { assert, test, readFileSync, join, packageRoot } = require("./context");
// Keep this snapshot in lockstep with every `export type` / `export interface` in the package `.d.ts` files.
const expectedTypeNames = require("./exported-type-names.json");

function publicDeclarationFiles() {
  return readdirSync(packageRoot)
    .filter((name) => name.endsWith(".d.ts"))
    .sort();
}

function declarationSource(name) {
  return readFileSync(join(packageRoot, name), "utf8");
}

function namedDeclarations(source) {
  return source.matchAll(/^(export )?(type|interface) (\w+)/gm);
}

test("public declaration files export every named type and interface", () => {
  const unexported = [];
  for (const name of publicDeclarationFiles()) {
    for (const match of namedDeclarations(declarationSource(name))) {
      if (!match[1]) unexported.push(`${name}:${match[3]}`);
    }
  }
  assert.deepEqual(unexported, []);
});

test("indexed access in public declarations is only a named export alias", () => {
  const bare = [];
  for (const name of publicDeclarationFiles()) {
    const source = declarationSource(name);
    for (const match of source.matchAll(/\b([A-Z]\w+)\s*\[\s*(["'])(\w+)\2\s*\]/g)) {
      const before = source.slice(0, match.index);
      if (!/export type \w+\s*=\s*$/.test(before)) {
        bare.push(`${name}:${match[0]}`);
      }
    }
  }
  assert.deepEqual(bare, []);
});

test("exported type and interface names stay locked", () => {
  const names = [];
  for (const name of publicDeclarationFiles()) {
    for (const match of namedDeclarations(declarationSource(name))) {
      if (match[1]) names.push(match[3]);
    }
  }
  names.sort();
  assert.deepEqual(names, expectedTypeNames);
});
