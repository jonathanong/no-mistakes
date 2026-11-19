"use strict";

const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");
const { isImportedHelper, localFunction } = require("./postgres-no-executor-adapter-target");
const { literalSql, tsLiteralSql } = require("./postgres-no-executor-adapter-sql");
const { localHoleSafe, tsHoleSafe } = require("./postgres-no-executor-adapter-sql-origin");
const { localExecutorType } = require("./postgres-no-executor-adapter-type");
const { passedOn } = require("./postgres-no-executor-adapter-option-forward");

function opaqueBody(body) {
  const stack = [body];
  while (stack.length) {
    const node = stack.pop();
    if (node.type === "Identifier" && node.name === "arguments") return true;
    const callee = node.type === "CallExpression" ? unwrapTs(node.callee) : null;
    if (callee?.type === "Identifier" && callee.name === "eval") return true;
    stack.push(...childNodes(node));
  }
  return false;
}

function parameterBinding(param, property) {
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  if (!property) return target?.type === "Identifier" ? { node: target, object: false } : null;
  if (target?.type === "Identifier") return { node: target, object: true };
  if (target?.type !== "ObjectPattern") return null;
  const field = target.properties.find(
    (entry) =>
      entry.type === "Property" &&
      !entry.computed &&
      entry.key.name === property &&
      entry.value.type === "Identifier",
  );
  return field ? { node: field.value, object: false } : null;
}

function executorCall(reference, property) {
  let node = reference.identifier;
  while (node.parent?.type === "TSNonNullExpression") node = node.parent;
  let callee = node;
  if (property && node.parent?.type === "MemberExpression" && node.parent.object === node) {
    if (node.parent.computed || node.parent.property.name !== property) return null;
    callee = node.parent;
  }
  const call = callee.parent;
  return call?.type === "CallExpression" && call.callee === callee ? call : null;
}

function otherPropertyRead(reference, property) {
  const member = reference.identifier.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== reference.identifier ||
    member.computed ||
    member.property.name === property
  )
    return false;
  const parent = member.parent;
  if (parent?.type === "AssignmentExpression" && parent.left === member) return false;
  if (
    parent?.type === "UpdateExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "delete")
  )
    return false;
  return true;
}

function importedVisible(call, index, property, context, options, facts) {
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return null;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const callee = services.esTreeNodeToTSNodeMap.get(call.callee);
  const signatures = checker.getTypeAtLocation(callee).getCallSignatures();
  if (signatures.length !== 1) return null;
  const declaration = signatures[0].getDeclaration();
  const parameter = declaration?.parameters?.[index];
  if (!declaration?.body || !parameter) return null;
  let binding = parameter.name;
  let object = false;
  if (property && ts.isObjectBindingPattern(binding)) {
    binding = binding.elements.find(
      (element) =>
        ts.isIdentifier(element.name) &&
        (element.propertyName?.text ?? element.name.text) === property &&
        !element.initializer &&
        !element.dotDotDotToken,
    )?.name;
  } else if (property) {
    object = true;
  }
  if (!binding || !ts.isIdentifier(binding)) return null;
  const symbol = checker.getSymbolAtLocation(binding);
  if (!symbol) return null;
  let seen = false;
  let safe = true;
  function visit(node) {
    if (!safe) return;
    if (ts.isIdentifier(node) && node.text === "arguments") safe = false;
    if (node !== binding && ts.isIdentifier(node) && checker.getSymbolAtLocation(node) === symbol) {
      seen = true;
      let invoked = node.parent;
      if (
        object &&
        ts.isPropertyAccessExpression(invoked) &&
        invoked.expression === node &&
        invoked.name.text !== property
      ) {
        const parent = invoked.parent;
        if (
          (ts.isBinaryExpression(parent) &&
            parent.left === invoked &&
            ts.isAssignmentOperator(parent.operatorToken.kind)) ||
          ts.isPrefixUnaryExpression(parent) ||
          ts.isPostfixUnaryExpression(parent) ||
          ts.isDeleteExpression(parent)
        )
          safe = false;
        return;
      }
      if (
        object &&
        ts.isPropertyAccessExpression(invoked) &&
        invoked.expression === node &&
        invoked.name.text === property
      ) {
        invoked = invoked.parent;
      }
      if (object && ts.isCallExpression(invoked) && invoked.arguments.includes(node)) return;
      if (
        !ts.isCallExpression(invoked) ||
        (invoked.expression !== node && invoked.expression !== node.parent) ||
        !invoked.arguments[0] ||
        !tsLiteralSql(invoked.arguments[0], checker, ts, options, (hole) =>
          tsHoleSafe(
            hole,
            call,
            declaration,
            checker,
            ts,
            context,
            facts,
            services.program,
            symbol,
            property,
          ),
        )
      ) {
        safe = false;
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(declaration.body);
  return seen && safe;
}

function visibleSqlTarget(call, index, quoteNode, context, options, accepted, facts) {
  if (call.type !== "CallExpression") return null;
  if (call.arguments.some((argument) => argument.type === "SpreadElement")) return null;
  if (quoteNode && (quoteNode.type !== "Property" || quoteNode.computed)) return null;
  const fn = localFunction(call.callee, context, facts);
  if (!fn)
    return accepted && isImportedHelper(call.callee, context)
      ? (importedVisible(call, index, quoteNode?.key.name, context, options, facts) ?? false)
      : null;
  if (fn.params.some((param) => param.type === "RestElement")) return false;
  const property = quoteNode?.key.name;
  const param = fn.params[index];
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  if (
    !property &&
    target?.typeAnnotation &&
    !localExecutorType(target.typeAnnotation.typeAnnotation, context, options)
  )
    return false;
  const binding = parameterBinding(fn.params[index], property);
  if (!binding) return false;
  const variable = resolveVariable(binding.node, context);
  if (!variable?.references.length) return false;
  if (opaqueBody(fn.body)) return false;
  return variable.references.every((reference) => {
    if (reference.init) return true;
    if (reference.isWrite()) return false;
    if (binding.object && otherPropertyRead(reference, property)) return true;
    if (accepted && binding.object && passedOn(reference)) return true;
    const invocation = executorCall(reference, property);
    return Boolean(
      invocation &&
      literalSql(invocation.arguments[0], context, options, (hole) =>
        localHoleSafe(hole, call, fn, context, variable, property),
      ),
    );
  });
}

module.exports = { visibleSqlTarget };
