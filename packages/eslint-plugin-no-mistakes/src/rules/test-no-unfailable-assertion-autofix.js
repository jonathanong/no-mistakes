"use strict";

const { existsSync, readFileSync } = require("node:fs");
const { dirname, join, resolve } = require("node:path");
const { unwrap } = require("./test-message-expect");
const { immutableConstDef } = require("./test-no-unfailable-assertion-assertions");
const { getExpressionType } = require("./test-no-unfailable-assertion-types");

const COMMENT_OR_STRING_RE =
  /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\/\/[^\r\n]*|\/\*[\s\S]*?\*\//g;
const JEST_DOM_MODULE = /^@testing-library\/jest-dom(?:\/|$)/;
const JEST_DOM_IMPORT_RE =
  /(?:from\s+|import\s+|require\s*\(\s*)["']@testing-library\/jest-dom(?:\/[^"']*)?["']/;
const SETUP_FIELD_RE =
  /["']?(?:setupFiles|setupFilesAfterEnv)["']?\s*:\s*(\[[\s\S]*?\]|["'`][^"'`]+["'`])/g;
const QUOTED_STRING_RE = /["']([^"']+)["']/g;
const CONFIG_FILES = [
  "vitest.config.ts",
  "vitest.config.mts",
  "vitest.config.js",
  "vitest.config.mjs",
  "vitest.config.cjs",
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.js",
  "vite.config.mjs",
  "jest.config.ts",
  "jest.config.js",
  "jest.config.mjs",
  "jest.config.cjs",
  "jest.config.json",
  "package.json",
];
const SETUP_EXTENSIONS = ["", ".ts", ".tsx", ".mts", ".js", ".mjs", ".cjs"];
const MULTI_QUERY_RE = /^(?:get|find)AllBy[A-Z]/;
const FIND_QUERY_RE = /^find(?:All)?By[A-Z]/;

function isRequireJestDom(node) {
  const expr = unwrap(node);
  return (
    expr?.type === "CallExpression" &&
    expr.callee.type === "Identifier" &&
    expr.callee.name === "require" &&
    expr.arguments[0]?.type === "Literal" &&
    typeof expr.arguments[0].value === "string" &&
    JEST_DOM_MODULE.test(expr.arguments[0].value)
  );
}

function hasJestDomInFile(sourceCode) {
  return sourceCode.ast.body.some(
    (stmt) =>
      (stmt.type === "ImportDeclaration" && JEST_DOM_MODULE.test(stmt.source.value)) ||
      (stmt.type === "ExpressionStatement" && isRequireJestDom(stmt.expression)) ||
      (stmt.type === "VariableDeclaration" &&
        stmt.declarations.some((decl) => isRequireJestDom(decl.init))),
  );
}

function stripJsComments(source) {
  return source.replace(COMMENT_OR_STRING_RE, (match) => (match.startsWith("/") ? "" : match));
}

function extractSetupEntries(content) {
  const entries = [];
  for (const match of stripJsComments(content).matchAll(SETUP_FIELD_RE)) {
    for (const item of match[1].matchAll(QUOTED_STRING_RE)) entries.push(item[1]);
  }
  return entries;
}

function resolveSetupFile(configDir, entry) {
  const cleaned = entry.replace(/^<rootDir>[\\/]/, "");
  for (const ext of SETUP_EXTENSIONS) {
    const candidate = resolve(configDir, `${cleaned}${ext}`);
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

function hasJestDomInSetupFiles(filename) {
  if (!filename || filename === "<input>") return false;
  let dir = dirname(filename);
  for (let depth = 0; ; depth++) {
    let foundBoundary = false;
    for (const name of CONFIG_FILES) {
      const configPath = join(dir, name);
      if (!existsSync(configPath)) continue;
      foundBoundary = true;
      for (const entry of extractSetupEntries(readFileSync(configPath, "utf8"))) {
        if (JEST_DOM_MODULE.test(entry)) return true;
        const setupPath = resolveSetupFile(dir, entry);
        if (
          setupPath &&
          JEST_DOM_IMPORT_RE.test(stripJsComments(readFileSync(setupPath, "utf8")))
        ) {
          return true;
        }
      }
    }
    const parent = dirname(dir);
    if (foundBoundary || parent === dir || depth >= 5) return false;
    dir = parent;
  }
}

function hasJestDomInTypes(assertion, typeCtx) {
  if (!typeCtx) return false;
  const expectType = getExpressionType(assertion.expectCall, typeCtx, false);
  return Boolean(expectType?.getProperty("toBeInTheDocument"));
}

function isInAsyncFunction(node) {
  for (let current = node.parent; current; current = current.parent) {
    if (
      current.type === "FunctionDeclaration" ||
      current.type === "FunctionExpression" ||
      current.type === "ArrowFunctionExpression"
    ) {
      return Boolean(current.async);
    }
  }
  return false;
}

function isAwaitedQueryVar(node, context) {
  if (node.type !== "Identifier") return false;
  const decl = immutableConstDef(node, context);
  if (decl?.id.type !== "Identifier") return false;
  const init = unwrap(decl.init);
  return init.type === "AwaitExpression" || isAwaitedQueryVar(init, context);
}

function createThrowingQueryFix(node, assertion, query, context, typeCtx, cache) {
  const isMulti = MULTI_QUERY_RE.test(query);
  const isFind = FIND_QUERY_RE.test(query);
  const outerAwait = node.parent?.type === "AwaitExpression" ? node.parent : null;
  const outerExpr = outerAwait || node;
  const inStatement = outerExpr.parent?.type === "ExpressionStatement";
  const actualUnwrapped = unwrap(assertion.actual);
  const directAwait = actualUnwrapped.type === "AwaitExpression";
  const directCall =
    actualUnwrapped.type === "CallExpression" ||
    (directAwait && unwrap(actualUnwrapped.argument).type === "CallExpression");
  const isAwaitedValue =
    directAwait || (!directCall && isAwaitedQueryVar(actualUnwrapped, context));

  if (cache.fileOrSetup === null) {
    cache.fileOrSetup =
      hasJestDomInFile(context.sourceCode) || hasJestDomInSetupFiles(context.filename);
  }
  const hasJestDom = !isMulti && (cache.fileOrSetup || hasJestDomInTypes(assertion, typeCtx));
  const { sourceCode } = context;

  if (hasJestDom) {
    if (assertion.asyncModifier === "resolves") {
      if (!outerAwait && !isInAsyncFunction(node)) return null;
      const prefix = outerAwait ? "" : "await ";
      return (fixer) =>
        fixer.replaceText(
          node,
          `${prefix}${sourceCode.getText(assertion.expectCall)}.resolves.toBeInTheDocument()`,
        );
    }
    if (isFind && !isAwaitedValue) {
      if (!isInAsyncFunction(node)) return null;
      const before = sourceCode.text.slice(node.range[0], assertion.actual.range[0]);
      const after = sourceCode.text.slice(assertion.actual.range[1], assertion.expectCall.range[1]);
      return (fixer) =>
        fixer.replaceText(
          node,
          `${before}await ${sourceCode.getText(assertion.actual)}${after}.toBeInTheDocument()`,
        );
    }
    return (fixer) =>
      fixer.replaceTextRange(
        [assertion.expectCall.range[1], node.range[1]],
        ".toBeInTheDocument()",
      );
  }

  if (!inStatement || !directCall) return null;
  if (isFind) {
    if (directAwait) {
      return (fixer) => fixer.replaceText(outerExpr, sourceCode.getText(assertion.actual));
    }
    if (!outerAwait && !isInAsyncFunction(node)) return null;
    return (fixer) => fixer.replaceText(outerExpr, `await ${sourceCode.getText(assertion.actual)}`);
  }
  return (fixer) => fixer.replaceText(outerExpr, sourceCode.getText(assertion.actual));
}

module.exports = { createThrowingQueryFix };
