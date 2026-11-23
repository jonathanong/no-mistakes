"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { literalString } = require("./logical-call");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { importSpecifierName, requireSource } = require("./async-target-sources");

const RULE_NAME = "test-no-fixed-temp-paths";
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const TEMP_PATH_PATTERN = /^\/(?:var\/)?tmp\//;
const MODULES = new Set(["path", "os", "fs", "fs/promises"]);
const MEMBERS = new Set(["join", "tmpdir", "promises", "mkdtemp", "mkdtempSync"]);
const UNIQUE_DIRECTORY_METHODS = new Set([
  "fs.mkdtemp",
  "fs.mkdtempSync",
  "fs.promises.mkdtemp",
  "fs/promises.mkdtemp",
]);

function nodeModule(source) {
  const name = source?.replace(/^node:/, "");
  return MODULES.has(name) ? name : null;
}

function memberBinding(object, member) {
  return object && MEMBERS.has(member) ? `${object}.${member}` : null;
}

function importedBinding(definition) {
  const { node, parent } = definition;
  if (node.importKind === "type" || parent.importKind === "type") return null;
  if (parent.type === "TSImportEqualsDeclaration") {
    const reference = parent.moduleReference;
    return reference.type === "TSExternalModuleReference"
      ? nodeModule(reference.expression.value)
      : null;
  }
  const source = nodeModule(parent.source.value);
  if (!source || node.type !== "ImportSpecifier") return source;
  const name = importSpecifierName(node);
  return name === "default" ? source : memberBinding(source, name);
}

function destructuredMember(pattern, name) {
  const property = pattern.properties.find(
    (item) =>
      item.type === "Property" && item.value.type === "Identifier" && item.value.name === name,
  );
  if (!property) return null;
  return !property.computed && property.key.type === "Identifier"
    ? property.key.name
    : literalString(property.key);
}

function bindingResolver(context) {
  const bindings = new WeakMap();
  function resolve(input) {
    const node = unwrapExpression(input);
    if (node?.type === "CallExpression") return nodeModule(requireSource(node, context));
    if (node?.type === "MemberExpression") {
      const object = resolve(node.object);
      const member = memberPropertyName(node);
      return memberBinding(object, member);
    }
    if (node?.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    if (variable?.defs.length !== 1) return null;
    if (bindings.has(variable)) return bindings.get(variable);
    bindings.set(variable, null);
    const definition = variable.defs[0];
    let binding = null;
    if (definition.type === "ImportBinding") {
      binding = importedBinding(definition);
    } else if (definition.type === "Variable" && definition.parent.kind === "const") {
      const { id, init } = definition.node;
      const value = resolve(init);
      if (id.type === "Identifier") binding = value;
      if (value && id.type === "ObjectPattern") {
        binding = memberBinding(value, destructuredMember(id, node.name));
      }
    }
    bindings.set(variable, binding);
    return binding;
  }
  return resolve;
}

function isUniqueDirectoryPrefix(node, resolve) {
  const argument = unwrapTransparentParent(node);
  const call = argument.parent;
  return (
    call?.type === "CallExpression" &&
    call.arguments[0] === argument &&
    UNIQUE_DIRECTORY_METHODS.has(resolve(call.callee))
  );
}

function isFixedTempJoin(node, resolve) {
  const [root, ...segments] = node.arguments;
  const directory = unwrapExpression(root);
  return (
    directory?.type === "CallExpression" &&
    directory.arguments.length === 0 &&
    resolve(node.callee) === "path.join" &&
    resolve(directory.callee) === "os.tmpdir" &&
    segments.every((segment) => literalString(unwrapExpression(segment)) !== null)
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
    function report(node) {
      if (!suppressed(node) && !isUniqueDirectoryPrefix(node, resolve)) {
        context.report({ node, messageId: "fixedTempPath" });
      }
    }
    function checkLiteral(node) {
      if (node.parent.type === "TaggedTemplateExpression" && node.parent.quasi === node) return;
      const value = literalString(node);
      if (value !== null && TEMP_PATH_PATTERN.test(value)) report(node);
    }
    return {
      Literal: checkLiteral,
      TemplateLiteral: checkLiteral,
      CallExpression(node) {
        if (isFixedTempJoin(node, resolve)) report(node);
      },
    };
  },
);
