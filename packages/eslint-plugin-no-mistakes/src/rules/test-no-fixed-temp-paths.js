"use strict";

const { posix } = require("node:path");
const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { stableConstInitializer } = require("./async-target-bindings");
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

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require unique temporary paths in tests",
      recommended: false,
    },
    schema: [{ type: "object", properties: { unanalyzable: UNANALYZABLE_SCHEMA } }],
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
    const advisories = new Set();
    let mocksFilesystem = false;
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
    function recordPath(node) {
      if (isFixedTempPath(node, resolve, origin) && !suppressed(node)) findings.add(node);
    }
    return {
      CallExpression(node) {
        if (mocksFilesystem) return;
        const binding = resolve(node.callee);
        if (binding === UNRESOLVED) {
          if (!suppressed(node)) advisories.add(node);
          return;
        }
        if (isFilesystemMock(node, binding)) {
          mocksFilesystem = true;
          return;
        }
        forEachPathArgument(node.arguments, binding, origin, recordPath);
      },
      "Program:exit"() {
        if (mocksFilesystem) return;
        for (const node of advisories) reportUnanalyzable(context, node, "this call");
        for (const node of findings) context.report({ node, messageId: "fixedTempPath" });
      },
    };
  },
);
