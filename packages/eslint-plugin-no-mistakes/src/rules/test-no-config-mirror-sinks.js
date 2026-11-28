"use strict";

const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, stableConstInitializer } = require("./async-target-bindings");
const { literalString } = require("./logical-call");
const { expectArguments, isExpectCall, memberName, unwrap } = require("./test-message-expect");
const { isUnshadowedGlobal } = require("./test-no-fixed-temp-paths-bindings");

const SINK_MATCHERS = new Set(["toContain", "toMatch", "toEqual", "toBe", "toStrictEqual"]);
const NEUTRAL_MATCHERS = new Set([
  "toBeDefined",
  "toBeTruthy",
  "toBeFalsy",
  "toBeUndefined",
  "toBeNull",
]);
const MODIFIERS = new Set(["not", "resolves", "rejects"]);
const WRITE_METHODS = new Set([
  "fs.writeFileSync",
  "fs.writeFile",
  "fs.appendFileSync",
  "fs.appendFile",
  "fs.copyFileSync",
  "fs.copyFile",
  "fs.createWriteStream",
  "fs.promises.writeFile",
  "fs.promises.appendFile",
  "fs.promises.copyFile",
  "fs/promises.writeFile",
  "fs/promises.appendFile",
  "fs/promises.copyFile",
  "fs-extra.writeFileSync",
  "fs-extra.writeFile",
  "fs-extra.writeJsonSync",
  "fs-extra.writeJson",
  "fs-extra.outputFileSync",
  "fs-extra.outputFile",
]);
const EXEC_ARGV_METHODS = new Set([
  "child_process.spawn",
  "child_process.spawnSync",
  "child_process.execFile",
  "child_process.execFileSync",
  "child_process.fork",
]);
const EXEC_SHELL_METHODS = new Set(["child_process.exec", "child_process.execSync"]);

function isStaticLiteral(input, context, seen = new Set()) {
  const node = unwrapExpression(input);
  if (!node) return false;
  if (node.type === "Literal") return true;
  if (node.type === "TemplateLiteral") {
    return node.expressions.every((expr) => isStaticLiteral(expr, context, seen));
  }
  if (node.type === "Identifier") {
    if (isUnshadowedGlobal(node, "undefined", context)) return true;
    const init = stableConstInitializer(node, context, new Set(seen));
    return init ? isStaticLiteral(init, context, seen) : false;
  }
  if (node.type === "UnaryExpression") {
    return (
      ["void", "-", "+", "!", "~"].includes(node.operator) &&
      isStaticLiteral(node.argument, context, seen)
    );
  }
  if (node.type === "ArrayExpression") {
    return node.elements.every(
      (el) => !el || isStaticLiteral(el.type === "SpreadElement" ? el.argument : el, context, seen),
    );
  }
  if (node.type === "ObjectExpression") {
    return node.properties.every((prop) => {
      if (prop.type === "SpreadElement") return isStaticLiteral(prop.argument, context, seen);
      return (
        prop.kind === "init" &&
        !prop.method &&
        (!prop.computed || isStaticLiteral(prop.key, context, seen)) &&
        isStaticLiteral(prop.value, context, seen)
      );
    });
  }
  if (
    (node.type === "NewExpression" || node.type === "CallExpression") &&
    isUnshadowedGlobal(unwrapExpression(node.callee), "RegExp", context)
  ) {
    return (
      node.arguments.length >= 1 &&
      node.arguments.every(
        (arg) => arg.type !== "SpreadElement" && isStaticLiteral(arg, context, seen),
      )
    );
  }
  return false;
}

function writePathArgument(node, binding) {
  if (!binding || !WRITE_METHODS.has(binding)) return null;
  const index = binding.endsWith(".copyFile") || binding.endsWith(".copyFileSync") ? 1 : 0;
  return node.arguments[index] ?? null;
}

function collectExecutedArguments(node, binding, context, visit) {
  if (EXEC_ARGV_METHODS.has(binding)) {
    const [cmd, argv] = node.arguments;
    if (cmd?.type !== "SpreadElement") visit(cmd);
    const argsNode = unwrapContainer(argv, context);
    if (argsNode?.type === "ArrayExpression") {
      for (const el of argsNode.elements) {
        if (el?.type !== "SpreadElement") visit(el);
      }
    }
  } else if (EXEC_SHELL_METHODS.has(binding)) {
    const cmd = unwrapExpression(node.arguments[0]);
    if (cmd?.type === "TemplateLiteral") {
      for (const expr of cmd.expressions) visit(expr);
    } else {
      visit(cmd);
    }
  }
}

function unwrapContainer(input, context) {
  const node = unwrapExpression(input);
  if (node?.type !== "Identifier") return node;
  const init = stableConstInitializer(node, context, new Set());
  return init ? unwrapExpression(init) : node;
}

function classifyExpectCall(callNode, context) {
  const callee = unwrap(callNode.callee);
  const matcher = callee?.type === "MemberExpression" ? memberName(callee) : null;
  if (!matcher) return null;
  let obj = unwrap(callee.object);
  while (obj?.type === "MemberExpression" && MODIFIERS.has(memberName(obj))) {
    obj = unwrap(obj.object);
  }
  if (!isExpectCall(obj, context)) return null;
  const expectArgs = expectArguments(obj, context);
  const target = expectArgs[0];
  if (SINK_MATCHERS.has(matcher)) {
    const expected = callNode.arguments[0];
    const hasSpread = callNode.arguments.some((arg) => arg.type === "SpreadElement");
    const literal = !hasSpread && Boolean(expected) && isStaticLiteral(expected, context);
    return { kind: literal ? "mirror" : "escape", target, expectCall: obj, node: callNode };
  }
  if (NEUTRAL_MATCHERS.has(matcher) && callNode.arguments.length === 0) {
    return { kind: "neutral", target, expectCall: obj, node: callNode };
  }
  return { kind: "escape", target, expectCall: obj, node: callNode };
}

function isStaticPropertyAccess(memberNode) {
  if (!memberNode.computed) return true;
  const prop = unwrapExpression(memberNode.property);
  return (
    literalString(prop) !== null || (prop?.type === "Literal" && typeof prop.value === "number")
  );
}

function isDerivedMethodCall(callNode) {
  const callee = unwrapExpression(callNode.callee);
  const name = memberPropertyName(callee);
  if (name === "toString") {
    return callNode.arguments.length === 0 || literalString(callNode.arguments[0]) !== null;
  }
  return (
    callNode.arguments.length === 0 &&
    (name === "trim" || name === "trimStart" || name === "trimEnd" || name === "toJSON")
  );
}

function isObjectCollectionCall(callNode, argNode, context) {
  const callee = unwrapExpression(callNode.callee);
  if (callee?.type !== "MemberExpression") return false;
  const name = memberPropertyName(callee);
  return (
    (name === "keys" || name === "values" || name === "entries") &&
    isUnshadowedGlobal(unwrapExpression(callee.object), "Object", context) &&
    callNode.arguments.length === 1 &&
    unwrapExpression(callNode.arguments[0]) === argNode
  );
}

module.exports = {
  classifyExpectCall,
  collectExecutedArguments,
  isDerivedMethodCall,
  isObjectCollectionCall,
  isStaticPropertyAccess,
  writePathArgument,
};
