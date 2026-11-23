"use strict";

const { posix } = require("node:path");
const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { stableConstInitializer } = require("./async-target-bindings");
const { literalString } = require("./logical-call");
const { bindingResolver, isFilesystemMock } = require("./test-no-fixed-temp-paths-bindings");
const { forEachPathArgument } = require("./test-no-fixed-temp-paths-sinks");

const RULE_NAME = "test-no-fixed-temp-paths";
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const TEMP_PATH_PATTERN = /^\/(?:var\/)?tmp\//;
const TEMP_ROOTS = new Set(["/tmp", "/tmp/", "/var/tmp", "/var/tmp/"]);

function isFixedTempPath(node, resolve) {
  const value = literalString(node);
  if (value !== null) return TEMP_PATH_PATTERN.test(value);
  if (node?.type !== "CallExpression") return false;
  const binding = resolve(node.callee);
  if (binding !== "path.join" && binding !== "path.resolve") return false;

  const [root, ...segments] = node.arguments.map(unwrapExpression);
  const names = segments.map(literalString);
  if (names.includes(null) || !names.some(Boolean)) return false;

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

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require unique temporary paths in tests",
      recommended: false,
    },
    schema: [],
    messages: {
      fixedTempPath:
        "Fixed temporary paths can collide across test workers or runs. Create a unique directory with mkdtemp or mkdtempSync and use paths inside it.",
    },
  },
  (context) => {
    if (!TEST_FILE_PATTERN.test(context.filename)) return {};
    const suppressed = ruleSuppression(context, RULE_NAME);
    const resolve = bindingResolver(context);
    const findings = new Set();
    let mocksFilesystem = false;
    function origin(input) {
      const node = unwrapExpression(input);
      return unwrapExpression(stableConstInitializer(node, context, new Set())) ?? node;
    }
    function recordPath(node) {
      if (isFixedTempPath(node, resolve) && !suppressed(node)) findings.add(node);
    }
    return {
      CallExpression(node) {
        if (mocksFilesystem) return;
        const binding = resolve(node.callee);
        if (isFilesystemMock(node, binding)) {
          mocksFilesystem = true;
          return;
        }
        forEachPathArgument(node.arguments, binding, origin, recordPath);
      },
      "Program:exit"() {
        if (mocksFilesystem) return;
        for (const node of findings) context.report({ node, messageId: "fixedTempPath" });
      },
    };
  },
);
