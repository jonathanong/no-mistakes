"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { literalString, memberPropertyName } = require("./logical-call");
const { destructuredImport, importedBinding } = require("./postgres-no-async-sql-builder-import");

const RULE_ID = "postgres-pool-requires-error-listener";
const LISTENER_METHODS = new Set(["on", "once", "addListener"]);

function isPoolConstructor(node, context, modules) {
  const callee = unwrapExpression(node.callee);
  if (callee.type === "Identifier") {
    const binding =
      importedBinding(callee, context) ??
      destructuredImport(callee, context, { allowDefault: true });
    return binding?.name === "Pool" && modules.has(binding.source);
  }
  if (memberPropertyName(callee) !== "Pool") return false;
  const binding = importedBinding(unwrapExpression(callee.object), context);
  return (
    (binding?.kind === "default" || binding?.kind === "namespace") && modules.has(binding.source)
  );
}

function isErrorListenerReceiver(node) {
  const receiver = unwrapTransparentParent(node);
  const member = receiver.parent;
  if (member?.object !== receiver || !LISTENER_METHODS.has(memberPropertyName(member))) {
    return false;
  }
  const callee = unwrapTransparentParent(member);
  const call = callee.parent;
  return (
    call?.type === "CallExpression" &&
    call.callee === callee &&
    literalString(unwrapExpression(call.arguments[0])) === "error" &&
    Boolean(call.arguments[1] && call.arguments[1].type !== "SpreadElement")
  );
}

function poolBinding(node) {
  const value = unwrapTransparentParent(node);
  const owner = value.parent;
  if (owner?.type === "VariableDeclarator" && owner.init === value) return owner.id;
  if (owner?.type === "AssignmentExpression" && owner.operator === "=" && owner.right === value) {
    return owner.left;
  }
  return null;
}

function hasBoundErrorListener(node, context) {
  const binding = poolBinding(node);
  if (binding?.type !== "Identifier") return false;
  const variable = resolveVariable(binding, context);
  if (!variable) return false;
  if (
    variable.references.some((reference) => reference.isWrite() && reference.identifier !== binding)
  ) {
    return false;
  }
  return variable.references.some((reference) => isErrorListenerReceiver(reference.identifier));
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require an error listener on each PostgreSQL pool",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: { extraModules: { type: "array", items: { type: "string" } } },
        additionalProperties: false,
      },
    ],
    messages: {
      missingErrorListener:
        'Attach an "error" listener to this PostgreSQL pool in the module that creates it. An idle client error can otherwise crash the process.',
    },
  },
  (context) => {
    const modules = new Set(["pg", ...(context.options[0]?.extraModules ?? [])]);
    const suppressed = ruleSuppression(context, RULE_ID);
    return {
      NewExpression(node) {
        if (!isPoolConstructor(node, context, modules) || suppressed(node)) return;
        if (isErrorListenerReceiver(node) || hasBoundErrorListener(node, context)) return;
        context.report({ node, messageId: "missingErrorListener" });
      },
    };
  },
);
