"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { literalString, memberPropertyName } = require("./logical-call");
const { isPoolConstructor } = require("./postgres-pool-constructor");
const { isMemberWrite, memberEntry } = require("./postgres-pool-member");

const RULE_ID = "postgres-pool-requires-error-listener";
const LISTENER_METHODS = new Set(["on", "once", "addListener"]);

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
  if (owner?.type === "PropertyDefinition" && owner.value === value) return owner;
  if (owner?.type === "AssignmentExpression" && owner.operator === "=" && owner.right === value) {
    return owner.left;
  }
  return null;
}

function hasBoundErrorListener(binding, context) {
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
    const members = new Map();
    const candidates = [];
    return {
      ClassBody(node) {
        for (const definition of node.body) {
          const isField = definition.type === "PropertyDefinition" && !definition.declare;
          const isAccessor =
            definition.type === "AccessorProperty" ||
            (definition.type === "MethodDefinition" &&
              (definition.kind === "get" || definition.kind === "set"));
          if (!isField && !isAccessor) continue;
          const member = memberEntry(members, definition);
          if (!member) continue;
          if (isField) {
            member.lastField = definition;
            if (definition.value) member.writes += 1;
          }
          if (isAccessor) member.usesAccessor = true;
        }
      },
      MemberExpression(node) {
        if (unwrapExpression(node.object)?.type !== "ThisExpression") return;
        const isWrite = isMemberWrite(node);
        const hasListener = isErrorListenerReceiver(node);
        if (!isWrite && !hasListener) return;
        const member = memberEntry(members, node);
        if (!member) return;
        if (isWrite) member.writes += 1;
        if (hasListener) member.hasListener = true;
      },
      NewExpression(node) {
        if (isPoolConstructor(node, context, modules) && !suppressed(node)) candidates.push(node);
      },
      "Program:exit"() {
        for (const node of candidates) {
          if (isErrorListenerReceiver(node)) continue;
          const binding = poolBinding(node);
          if (binding?.type === "Identifier" && hasBoundErrorListener(binding, context)) continue;
          const member = binding && memberEntry(members, binding);
          if (
            member?.writes === 1 &&
            member.hasListener &&
            !member.usesAccessor &&
            (binding.type !== "PropertyDefinition" || member.lastField === binding)
          ) {
            continue;
          }
          context.report({ node, messageId: "missingErrorListener" });
        }
      },
    };
  },
);
