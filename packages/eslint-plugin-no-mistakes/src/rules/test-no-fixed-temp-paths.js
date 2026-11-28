"use strict";

const { posix } = require("node:path");
const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { requireResolution } = require("./async-target-sources");
const { literalString } = require("./logical-call");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const {
  UNRESOLVED,
  bindingResolver,
  isFilesystemMock,
} = require("./test-no-fixed-temp-paths-bindings");
const { forEachPathArgument } = require("./test-no-fixed-temp-paths-sinks");

const RULE_NAME = "test-no-fixed-temp-paths";
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const TEMP_PATH_PATTERN = /^\/(?:var\/)?tmp\//;
const TEMP_ROOTS = new Set(["/tmp", "/tmp/", "/var/tmp", "/var/tmp/"]);

function isFixedTempPath(node, resolve, origin) {
  const value = literalString(node);
  if (value !== null) return TEMP_PATH_PATTERN.test(value);
  if (node?.type !== "CallExpression") return false;
  const binding = resolve(node.callee);
  if (binding !== "path.join" && binding !== "path.resolve") return false;

  const [rootArgument, ...segments] = node.arguments;
  const names = segments.map((segment) => literalString(unwrapExpression(segment)));
  if (names.includes(null) || !names.some(Boolean)) return false;

  const root = origin(rootArgument);
  const literalRoot = literalString(root);
  if (TEMP_ROOTS.has(literalRoot) || TEMP_PATH_PATTERN.test(literalRoot ?? "")) {
    const method = binding === "path.join" ? posix.join : posix.resolve;
    return TEMP_PATH_PATTERN.test(method(literalRoot, ...names));
  }
  if (
    root?.type !== "CallExpression" ||
    root.arguments.length ||
    resolve(root.callee) !== "os.tmpdir"
  ) {
    return false;
  }
  return (
    binding === "path.join" ||
    !names.some(posix.isAbsolute) ||
    TEMP_PATH_PATTERN.test(posix.resolve(...names))
  );
}

function reachedThroughRequire(node, context, seen = new Set()) {
  let current = unwrapExpression(node?.type === "CallExpression" ? node.callee : node);
  while (current) {
    if (current.type === "CallExpression" && requireResolution(current, context)) return true;
    if (current.type === "MemberExpression") {
      current = unwrapExpression(current.object);
      continue;
    }
    if (current.type !== "Identifier") return false;
    const variable = resolveVariable(current, context);
    if (!variable || seen.has(variable)) return false;
    seen.add(variable);
    const def = variable.defs.length === 1 ? variable.defs[0] : null;
    // `import fs = require("node:fs")` is a require(). An ordinary import is not.
    if (
      def?.type === "ImportBinding" &&
      def.parent?.type === "TSImportEqualsDeclaration" &&
      def.parent.moduleReference?.type === "TSExternalModuleReference"
    ) {
      return true;
    }
    if (def?.type !== "Variable" || def.parent?.kind !== "const") return false;
    current = unwrapExpression(def.node?.init);
  }
  return false;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require unique temporary paths in tests",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        additionalProperties: false,
        properties: { unanalyzable: UNANALYZABLE_SCHEMA },
      },
    ],
    messages: {
      fixedTempPath:
        "Fixed temporary paths can collide across test workers or runs. Create a unique directory with mkdtemp or mkdtempSync and use paths inside it.",
      unanalyzable:
        "Cannot tell whether {{subject}} writes a fixed temporary path. Use a static fs or path binding.",
    },
  },
  (context) => {
    if (!TEST_FILE_PATTERN.test(context.filename)) return {};
    const suppressed = ruleSuppression(context, RULE_NAME);
    const resolve = bindingResolver(context);
    const findings = new Set();
    const requireFindings = new Set();
    const advisories = new Set();
    let vitestMocksFs = false;
    let jestMocksFs = false;
    function origin(input) {
      const seen = new Set();
      let node = unwrapExpression(input);
      while (node?.type === "Identifier") {
        const initializer = stableConstInitializer(node, context, seen);
        if (!initializer) break;
        node = unwrapExpression(initializer);
      }
      return node;
    }
    function recordPath(node, call) {
      if (!isFixedTempPath(node, resolve, origin) || suppressed(node)) return;
      findings.add(node);
      if (reachedThroughRequire(call, context)) requireFindings.add(node);
    }
    return {
      CallExpression(node) {
        const binding = resolve(node.callee);
        if (binding === UNRESOLVED) {
          if (!suppressed(node)) advisories.add(node);
          return;
        }
        if (isFilesystemMock(node, binding)) {
          if (binding === "vitest.vi.mock") vitestMocksFs = true;
          else jestMocksFs = true;
          return;
        }
        forEachPathArgument(node.arguments, binding, origin, (path) => recordPath(path, node));
      },
      "Program:exit"() {
        if (jestMocksFs) return;
        const reportFinding = (node) => context.report({ node, messageId: "fixedTempPath" });
        if (!vitestMocksFs) {
          for (const node of advisories) reportUnanalyzable(context, node, "this call");
          for (const node of findings) reportFinding(node);
          return;
        }
        for (const node of advisories) {
          if (reachedThroughRequire(node, context)) reportUnanalyzable(context, node, "this call");
        }
        for (const node of requireFindings) reportFinding(node);
      },
    };
  },
);
