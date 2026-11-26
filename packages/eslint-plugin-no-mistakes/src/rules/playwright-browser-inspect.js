"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

const CALL_WRAPPERS = new Set(["call", "apply", "bind"]);
const PURE_UNARY = new Set(["!", "typeof", "void"]);
const PURE_BINARY = new Set(["===", "!==", "==", "!=", "instanceof"]);
const TEST_PARENTS = new Set([
  "IfStatement",
  "WhileStatement",
  "DoWhileStatement",
  "ForStatement",
  "ConditionalExpression",
]);

function hasLaterWrite(variable) {
  const writes = variable.references.filter((ref) => ref.isWrite());
  return writes.length > 1 || writes.some((ref) => !ref.init);
}

function isPureCheck(expr, parent) {
  return (
    parent.type === "TSTypeQuery" ||
    (parent.type === "UnaryExpression" && PURE_UNARY.has(parent.operator)) ||
    (parent.type === "BinaryExpression" && PURE_BINARY.has(parent.operator)) ||
    (TEST_PARENTS.has(parent.type) && parent.test === expr) ||
    (parent.type === "LogicalExpression" &&
      parent.left === expr &&
      (parent.operator === "&&" || parent.operator === "||") &&
      (parent.parent.type === "ExpressionStatement" || isPureCheck(parent, parent.parent)))
  );
}

function isBoundReceiverArg(expr, parent, bindings, context) {
  if (
    parent.type !== "CallExpression" ||
    parent.arguments[0] !== expr ||
    parent.callee.type !== "MemberExpression" ||
    !CALL_WRAPPERS.has(bindings.memberKey(parent.callee)) ||
    parent.callee.object.type !== "MemberExpression"
  ) {
    return false;
  }
  const receiver = parent.callee.object.object;
  return (
    expr.type === "Identifier" &&
    receiver.type === "Identifier" &&
    resolveVariable(expr, context) === resolveVariable(receiver, context)
  );
}

function createBrowserInspector(
  context,
  bindings,
  browserFixtureName,
  reportNewPage,
  reportUnanalyzable,
) {
  const visitedFixtures = new Set();
  const visitedBrowsers = new Set();

  function inspectNewPageVariable(variable, fallbackNode) {
    const reads = hasLaterWrite(variable) ? [] : variable.references.filter((ref) => ref.isRead());
    if (reads.length === 0) {
      reportNewPage(fallbackNode);
      return;
    }
    for (const ref of reads) inspectNewPageExpr(ref.identifier);
  }

  function inspectNewPageExpr(node) {
    const expr = unwrapTransparentParent(node);
    const parent = expr.parent;
    if (parent.type === "CallExpression" && parent.callee === expr) {
      reportNewPage(parent);
      return;
    }
    if (parent.type === "MemberExpression" && parent.object === expr) {
      const method = bindings.memberKey(parent);
      if (CALL_WRAPPERS.has(method)) {
        const outer = unwrapTransparentParent(parent);
        if (outer.parent.type === "CallExpression" && outer.parent.callee === outer) {
          if (method === "bind") inspectNewPageExpr(outer.parent);
          else reportNewPage(outer.parent);
          return;
        }
        reportNewPage(parent);
        return;
      }
    }
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === expr &&
      parent.id.type === "Identifier"
    ) {
      inspectNewPageVariable(resolveVariable(parent.id, context), parent);
      return;
    }
    reportNewPage(node);
  }

  function inspectBrowserPattern(pattern) {
    for (const prop of pattern.properties) {
      if (prop.type === "RestElement") {
        reportUnanalyzable(prop, `rest destructuring on \`${browserFixtureName}\``);
        continue;
      }
      const key = bindings.propertyKey(prop);
      if (key === null) {
        reportUnanalyzable(prop, `dynamic property destructuring on \`${browserFixtureName}\``);
        continue;
      }
      if (key !== "newPage") continue;
      const value = prop.value.type === "AssignmentPattern" ? prop.value.left : prop.value;
      if (value.type === "Identifier")
        inspectNewPageVariable(resolveVariable(value, context), prop);
      else reportNewPage(prop);
    }
  }

  function inspectObjectExpr(node, target) {
    const expr = unwrapTransparentParent(node);
    const parent = expr.parent;
    if (parent.type === "MemberExpression" && parent.object === expr) {
      const key = bindings.memberKey(parent);
      if (key === null) reportUnanalyzable(parent, `dynamic member access on ${target.label}`);
      else if (key === target.member) target.onMember(parent);
      return;
    }
    if (parent.type === "VariableDeclarator" && parent.init === expr) {
      if (parent.id.type === "Identifier") target.onAlias(resolveVariable(parent.id, context));
      else if (parent.id.type === "ObjectPattern") target.onPattern(parent.id);
      else reportUnanalyzable(parent.id, `destructuring ${target.label}`);
      return;
    }
    if (
      !isPureCheck(expr, parent) &&
      !(target.allowBoundThis && isBoundReceiverArg(expr, parent, bindings, context))
    ) {
      reportUnanalyzable(node, `escaping ${target.label}`);
    }
  }

  function inspectVariable(variable, visited, target) {
    if (!variable || visited.has(variable)) return;
    visited.add(variable);
    if (hasLaterWrite(variable)) {
      reportUnanalyzable(variable.identifiers[0], `reassigned ${target.label} binding`);
      return;
    }
    for (const ref of variable.references) {
      if (ref.isRead()) inspectObjectExpr(ref.identifier, target);
    }
  }

  const browserTarget = {
    label: `\`${browserFixtureName}\``,
    member: "newPage",
    allowBoundThis: true,
    onMember: inspectNewPageExpr,
    onAlias: (variable) => inspectVariable(variable, visitedBrowsers, browserTarget),
    onPattern: inspectBrowserPattern,
  };

  function inspectFixturesPattern(pattern) {
    const hasBrowser = pattern.properties.some(
      (prop) => prop.type === "Property" && bindings.propertyKey(prop) === browserFixtureName,
    );
    for (const prop of pattern.properties) {
      if (prop.type === "RestElement") {
        if (!hasBrowser) reportUnanalyzable(prop, "rest fixture destructuring");
        continue;
      }
      const key = bindings.propertyKey(prop);
      if (key === null) {
        reportUnanalyzable(prop, "dynamic fixture destructuring");
        continue;
      }
      if (key !== browserFixtureName) continue;
      const value = prop.value.type === "AssignmentPattern" ? prop.value.left : prop.value;
      if (value.type === "Identifier") browserTarget.onAlias(resolveVariable(value, context));
      else if (value.type === "ObjectPattern") inspectBrowserPattern(value);
      else reportUnanalyzable(value, `destructuring \`${browserFixtureName}\``);
    }
  }

  const fixturesTarget = {
    label: "Playwright fixtures",
    member: browserFixtureName,
    allowBoundThis: false,
    onMember: (node) => inspectObjectExpr(node, browserTarget),
    onAlias: (variable) => inspectVariable(variable, visitedFixtures, fixturesTarget),
    onPattern: inspectFixturesPattern,
  };

  return function inspectCallback(fn) {
    const first = fn.params[0]?.type === "Identifier" && fn.params[0].name === "this" ? 1 : 0;
    let param = fn.params[first];
    if (!param) return;
    if (param.type === "RestElement") {
      reportUnanalyzable(param, "rest parameter capturing Playwright fixtures");
      return;
    }
    if (param.type === "AssignmentPattern") param = param.left;
    if (param.type === "ObjectPattern") inspectFixturesPattern(param);
    else if (param.type === "Identifier") fixturesTarget.onAlias(resolveVariable(param, context));
    else reportUnanalyzable(param, "destructuring Playwright fixtures");
  };
}

module.exports = { createBrowserInspector };
