"use strict";

function parsedSource(filename, text, parser) {
  const parsed = parser.parseForESLint(text, { filePath: filename, range: true, loc: true });
  function parents(node, parent) {
    node.parent = parent;
    for (const key of parsed.visitorKeys[node.type] || []) {
      const child = node[key];
      if (Array.isArray(child)) child.forEach((item) => item && parents(item, node));
      else if (child) parents(child, node);
    }
  }
  parents(parsed.ast, null);
  const sourceCode = {
    ...parsed,
    getScope(node) {
      // Every parsed node has a Program ancestor with a global scope.
      for (let owner = node; ; owner = owner.parent) {
        const scope = parsed.scopeManager.acquire(owner, true);
        if (scope) return scope;
      }
    },
  };
  return { ast: parsed.ast, context: { sourceCode, filename } };
}

function currentSources(context, store) {
  const filename = context.filename;
  let main;
  return {
    resolve: store.resolve,
    load(path) {
      // Oxlint reuses its Context/SourceCode/AST. Keep this overlay per file;
      // the shared store contains only independently parsed imported sources.
      return path === filename
        ? (main ||= { ast: context.sourceCode.ast, context })
        : store.load(path);
    },
  };
}

module.exports = { parsedSource, currentSources };
