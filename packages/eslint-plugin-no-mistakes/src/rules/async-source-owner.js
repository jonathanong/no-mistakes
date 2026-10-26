"use strict";

const { readFileSync, realpathSync } = require("node:fs");
const { dirname, resolve } = require("node:path");

// The public npm CLI calls native lint once. Editors, fix passes, and unknown
// hosts have no invocation boundary available to a JS plugin, so stay per-file.
function oneShotCli(entry = process.argv[1], args = process.argv.slice(2)) {
  if (!entry || args.some((arg) => /^--(?:lsp|watch)(?:=|$)/.test(arg) || arg.startsWith("--fix")))
    return false;
  try {
    const filename = realpathSync(entry);
    const root = resolve(dirname(filename), "..");
    const manifest = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
    return (
      manifest.name === "oxlint" &&
      typeof manifest.bin?.oxlint === "string" &&
      realpathSync(resolve(root, manifest.bin.oxlint)) === filename
    );
  } catch {
    return false;
  }
}

const activeOwners = new WeakMap();

function sourceOwner(context) {
  const owner = activeOwners.get(context);
  if (!owner) return null;
  owner.shared ??= oneShotCli();
  return owner.shared ? owner : null;
}

function sourcesUnavailable(context) {
  return activeOwners.get(context)?.sources === false;
}

function withSourceOwner(rule) {
  rule.createOnce = (context) => {
    const owner = { sources: null };
    let visitor;
    return {
      ...Object.fromEntries(
        [
          "Program",
          "ImportDeclaration",
          "TSImportEqualsDeclaration",
          "VariableDeclarator",
          "CallExpression",
        ].map((key) => [key, (node) => visitor[key]?.(node)]),
      ),
      before() {
        activeOwners.set(context, owner);
        try {
          visitor = rule.create(context);
        } catch (error) {
          activeOwners.delete(context);
          throw error;
        }
      },
      after() {
        visitor = null;
        activeOwners.delete(context);
      },
    };
  };
  return rule;
}

module.exports = { oneShotCli, sourceOwner, sourcesUnavailable, withSourceOwner };
