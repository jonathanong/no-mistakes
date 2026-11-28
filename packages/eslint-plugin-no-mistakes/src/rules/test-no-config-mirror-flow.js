"use strict";

const { isFunction, unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { isUnshadowedGlobal } = require("./test-no-config-mirror-bindings");
const { enclosingTestBlock } = require("./test-no-config-mirror-scopes");
const {
  isDerivedMethodCall,
  isObjectCollectionCall,
  isStaticPropertyAccess,
} = require("./test-no-config-mirror-sinks");
const { isParserCallee } = require("./test-no-config-mirror-sources");

function patternVariables(pattern, context, out = []) {
  if (pattern?.type === "Identifier") {
    out.push(resolveVariable(pattern, context));
  } else if (pattern?.type === "AssignmentPattern") {
    patternVariables(pattern.left, context, out);
  } else if (pattern?.type === "ObjectPattern") {
    for (const prop of pattern.properties) {
      patternVariables(prop.type === "RestElement" ? prop.argument : prop.value, context, out);
    }
  } else if (pattern?.type === "ArrayPattern") {
    for (const el of pattern.elements) {
      patternVariables(el?.type === "RestElement" ? el.argument : el, context, out);
    }
  }
  return out;
}

function writeSourceExpression(reference) {
  const parent = reference.identifier.parent;
  if (parent.type === "AssignmentExpression") return parent.right;
  return parent.type === "VariableDeclarator" ? parent.init : null;
}

function isEmptyLiteral(node, context) {
  const expr = unwrapExpression(node);
  return (
    (expr.type === "Literal" && (expr.value === "" || expr.value === null)) ||
    isUnshadowedGlobal(expr, "undefined", context)
  );
}

function isPurelyTaintedVariable(variable, taintedWrites, context) {
  for (const def of variable.defs) {
    if (def.type === "Variable" && def.parent.kind === "let" && def.node.init) {
      if (!taintedWrites.has(def.node.init) && !isEmptyLiteral(def.node.init, context)) {
        return false;
      }
    }
  }
  for (const ref of variable.references) {
    if (!ref.isWrite()) continue;
    const assigned = writeSourceExpression(ref);
    if (assigned && !taintedWrites.has(assigned) && !isEmptyLiteral(assigned, context)) {
      return false;
    }
  }
  return true;
}

function analyzeSourceUsage({
  source,
  context,
  resolve,
  configuredParsers,
  expectMap,
  exemptTest,
}) {
  const visitedExprs = new Set();
  const visitedVars = new Set();
  const groupFindings = new Map();
  const escapedGroups = new Set();
  let moduleEscaped = false;

  const markEscape = (node) => {
    const block = enclosingTestBlock(node, context);
    if (!block) moduleEscaped = true;
    else escapedGroups.add(block);
  };

  const traceVariable = (variable, taintedWrites) => {
    if (!variable || visitedVars.has(variable)) return;
    if (!isPurelyTaintedVariable(variable, taintedWrites, context)) {
      return markEscape(variable.defs[0].node);
    }
    visitedVars.add(variable);
    for (const ref of variable.references) {
      if (ref.isRead()) traceExpression(ref.identifier);
    }
  };

  function traceExpression(expr) {
    if (visitedExprs.has(expr)) return;
    visitedExprs.add(expr);
    const wrapped = unwrapTransparentParent(expr);
    const parent = wrapped.parent;
    if (parent.type === "AwaitExpression") return traceExpression(parent);
    if (parent.type === "MemberExpression" && parent.object === wrapped) {
      return traceMember(parent);
    }
    if (parent.type === "VariableDeclarator" && parent.init === wrapped) {
      const writes = new Set([parent.init]);
      for (const v of patternVariables(parent.id, context)) traceVariable(v, writes);
      return;
    }
    if (
      parent.type === "AssignmentExpression" &&
      parent.right === wrapped &&
      parent.operator === "="
    ) {
      const writes = new Set([parent.right]);
      for (const v of patternVariables(parent.left, context)) traceVariable(v, writes);
      return;
    }
    if (
      parent.type === "LogicalExpression" &&
      (parent.operator === "??" || parent.operator === "||")
    ) {
      return traceExpression(parent);
    }
    if (parent.type === "CallExpression") return traceCall(parent, unwrapExpression(wrapped));
    markEscape(parent);
  }

  function traceMember(member) {
    const call =
      member.parent.type === "CallExpression" && member.parent.callee === member
        ? member.parent
        : null;
    if (call && memberPropertyName(member) === "then") {
      const cb = unwrapExpression(call.arguments[0]);
      if (isFunction(cb) && cb.params[0]) {
        for (const v of patternVariables(cb.params[0], context)) traceVariable(v, new Set());
        return;
      }
    }
    if (call) {
      return isDerivedMethodCall(call) ? traceExpression(call) : markEscape(call);
    }
    return isStaticPropertyAccess(member) ? traceExpression(member) : markEscape(member);
  }

  function traceCall(call, arg) {
    const classified = expectMap.get(call);
    if (classified && unwrapExpression(classified.target) === arg) {
      if (classified.kind === "mirror") {
        const block = enclosingTestBlock(classified.node, context);
        const list = groupFindings.get(block) ?? [];
        list.push(classified.node);
        groupFindings.set(block, list);
        return;
      }
      if (classified.kind === "neutral") return;
      return markEscape(classified.node);
    }
    if (
      (call.arguments[0] &&
        unwrapExpression(call.arguments[0]) === arg &&
        isParserCallee(unwrapExpression(call.callee), resolve, configuredParsers, context)) ||
      isObjectCollectionCall(call, arg, context)
    ) {
      return traceExpression(call);
    }
    markEscape(call);
  }

  if (source.callbackParam) {
    for (const v of patternVariables(source.callbackParam, context)) traceVariable(v, new Set());
  } else {
    traceExpression(source.node);
  }
  if (moduleEscaped) return [];
  const findings = [];
  for (const [block, nodes] of groupFindings.entries()) {
    if (escapedGroups.has(block) || exemptTest(block, source)) continue;
    for (const node of nodes) findings.push({ node, source });
  }
  return findings;
}

module.exports = { analyzeSourceUsage };
