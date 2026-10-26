"use strict";

const { sourceOwner } = require("./async-source-owner");
const { currentSources } = require("./async-source-files");
const { createSourceSession } = require("./async-source-session");

// Oxlint's type-aware checker does not expose a TypeScript program to JS plugins.
// A verified one-shot CLI owns shared imported facts; other hosts stay per-file.
function createImportedSources(context) {
  const owner = sourceOwner(context);
  if (owner?.sources === false) return null;
  if (owner?.sources) return currentSources(context, owner.sources);
  try {
    const ts = require("typescript");
    let parser;
    try {
      parser = require("@typescript-eslint/parser");
    } catch (error) {
      // A broken installed parser must not silently select a different proof model.
      if (
        error.code !== "MODULE_NOT_FOUND" ||
        !error.message.startsWith("Cannot find module '@typescript-eslint/parser'")
      )
        throw error;
      parser = {
        parseForESLint: (text, options) =>
          require("./async-source-ts-parser").parseTypeScript(ts, text, options.filePath),
      };
    }
    return createSources(context, ts, parser);
  } catch {
    // Optional tooling cannot establish a contract when unavailable or incompatible.
    if (owner) owner.sources = false;
    return null;
  }
}

function createSources(context, ts, parser) {
  const store = createSourceSession(ts, parser);
  const owner = sourceOwner(context);
  if (owner) owner.sources = store || false;
  return store && currentSources(context, store);
}

module.exports = { createImportedSources };
