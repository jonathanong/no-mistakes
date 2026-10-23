"use strict";
const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { importSelectorSchema, matchesRuntimeImport } = require("./configured-runtime-import");
const { ruleSuppression } = require("../rule-suppression");
const strings = { type: "array", items: { type: "string", minLength: 1 }, uniqueItems: true };
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "batch adjacent independent reads on explicitly configured clients",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          clients: strings,
          imports: { type: "array", items: importSelectorSchema },
          readMethods: strings,
          message: { type: "string", minLength: 1 },
        },
        required: ["readMethods"],
        additionalProperties: false,
      },
    ],
    messages: { unbatchedReads: "{{message}}" },
  },
  create(context) {
    const options = context.options[0] ?? {};
    const suppressed = ruleSuppression(context, "no-consecutive-unbatched-reads");
    function read(statement) {
      let expression;
      if (statement.type === "ExpressionStatement") expression = statement.expression;
      if (statement.type === "VariableDeclaration" && statement.declarations.length === 1)
        expression = statement.declarations[0].init;
      const awaited = unwrapExpression(expression);
      if (awaited?.type !== "AwaitExpression") return null;
      const call = unwrapExpression(awaited.argument);
      if (call.type !== "CallExpression") return null;
      const member = unwrapExpression(call.callee);
      if (
        member.type !== "MemberExpression" ||
        unwrapExpression(member.object).type !== "Identifier" ||
        !options.readMethods?.includes(memberPropertyName(member))
      )
        return null;
      const client = unwrapExpression(member.object);
      if (
        !options.clients?.includes(client.name) &&
        !options.imports?.some((selector) => matchesRuntimeImport(client, selector, context))
      )
        return null;
      return { call, client, variable: resolveVariable(client, context) };
    }
    function check(node) {
      for (let index = 1; index < node.body.length; index++) {
        const previous = node.body[index - 1];
        const current = node.body[index];
        const first = read(previous);
        const second = read(current);
        if (
          !first ||
          !second ||
          first.variable !== second.variable ||
          first.client.name !== second.client.name ||
          suppressed(current)
        )
          continue;
        // Reference identity covers destructuring and avoids mistaking property names for dependencies.
        const variables =
          previous.type === "VariableDeclaration"
            ? context.sourceCode.getDeclaredVariables(previous)
            : [];
        const dependent = variables.some((variable) =>
          variable.references.some(
            ({ identifier }) =>
              identifier.range[0] >= second.call.range[0] &&
              identifier.range[1] <= second.call.range[1],
          ),
        );
        if (!dependent)
          context.report({
            node: current,
            messageId: "unbatchedReads",
            data: {
              message:
                options.message ??
                "Adjacent independent reads on the same client add round trips. Batch the complete key set with the client's batch API.",
            },
          });
      }
    }
    return { "Program:exit": check, "BlockStatement:exit": check };
  },
};
