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
      if (statement.type === "VariableDeclarator") expression = statement.init;
      if (statement.type === "ReturnStatement") expression = statement.argument;
      let assigned;
      expression = unwrapExpression(expression);
      if (expression?.type === "AssignmentExpression") {
        if (expression.operator !== "=") return null;
        assigned = expression.left;
        expression = expression.right;
      }
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
      const variables = assignedVariables(assigned);
      if (statement.type === "VariableDeclarator")
        variables.push(...context.sourceCode.getDeclaredVariables(statement));
      if (variables.includes(null)) return null;
      return { call, client, variables, variable: resolveVariable(client, context) };
    }
    function assignedVariables(pattern) {
      if (!pattern) return [];
      if (pattern.type === "Identifier") return [resolveVariable(pattern, context)];
      if (pattern.type === "RestElement") return assignedVariables(pattern.argument);
      if (pattern.type === "AssignmentPattern") return assignedVariables(pattern.left);
      if (pattern.type === "ArrayPattern") return pattern.elements.flatMap(assignedVariables);
      if (pattern.type === "ObjectPattern")
        return pattern.properties.flatMap((property) =>
          assignedVariables(property.type === "RestElement" ? property.argument : property.value),
        );
      // Property writes and unresolved targets have no sound result binding.
      return [null];
    }
    function check(statements) {
      const entries = statements.flatMap((statement) => {
        const node =
          statement.type === "ExportNamedDeclaration" ? statement.declaration : statement;
        return node?.type === "VariableDeclaration" ? node.declarations : [statement];
      });
      for (let index = 1; index < entries.length; index++) {
        const previous = entries[index - 1];
        const current = entries[index];
        // A return ends this sequence; unreachable reads cannot form a batch with it.
        if (previous.type === "ReturnStatement") continue;
        const first = read(previous);
        const second = read(current);
        // Preserve existing single-declaration diagnostic and suppression locations.
        const reportNode =
          current.type === "VariableDeclarator" && current.parent.declarations.length === 1
            ? current.parent
            : current;
        if (
          !first ||
          !second ||
          first.variable !== second.variable ||
          first.client.name !== second.client.name ||
          suppressed(reportNode) ||
          (current.type === "VariableDeclarator" &&
            (suppressed(current.parent) ||
              (current.parent.parent.type === "ExportNamedDeclaration" &&
                suppressed(current.parent.parent))))
        )
          continue;
        // Reference identity covers destructuring and avoids mistaking property names for dependencies.
        const dependent = first.variables.some((variable) =>
          variable.references.some(
            ({ identifier }) =>
              identifier.range[0] >= second.call.range[0] &&
              identifier.range[1] <= second.call.range[1],
          ),
        );
        if (!dependent)
          context.report({
            node: reportNode,
            messageId: "unbatchedReads",
            data: {
              message:
                options.message ??
                "Adjacent independent reads on the same client add round trips. Batch the complete key set with the client's batch API.",
            },
          });
      }
    }
    return {
      "Program:exit": (node) => check(node.body),
      "BlockStatement:exit": (node) => check(node.body),
      "SwitchCase:exit": (node) => check(node.consequent),
    };
  },
};
