"use strict";
const { unwrapExpression } = require("./async-ast");
const { importSelectorSchema, matchesRuntimeImport } = require("./configured-runtime-import");
const { ruleSuppression } = require("../rule-suppression");
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "require an imported registrar to receive a callback unconditionally at module top level",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          registrar: importSelectorSchema,
          callback: { type: "string", minLength: 1 },
          message: { type: "string", minLength: 1 },
        },
        required: ["registrar", "callback"],
        additionalProperties: false,
      },
    ],
    messages: { registration: "{{message}}" },
  },
  create(context) {
    const options = context.options[0];
    const suppressed = ruleSuppression(context, "require-top-level-registration");
    return {
      "Program:exit"(program) {
        if (!options) return;
        const registered = program.body.some((statement) => {
          if (statement.type !== "ExpressionStatement") return false;
          const call = unwrapExpression(statement.expression);
          if (
            call.type !== "CallExpression" ||
            call.optional ||
            call.callee.type !== "Identifier" ||
            call.arguments.length !== 1
          )
            return false;
          const callback = unwrapExpression(call.arguments[0]);
          return (
            callback.type === "Identifier" &&
            callback.name === options.callback &&
            matchesRuntimeImport(call.callee, options.registrar, context)
          );
        });
        const node = program.body[0] ?? program;
        if (!registered && !suppressed(node))
          context.report({
            node,
            messageId: "registration",
            data: {
              message:
                options.message ??
                "Import the configured runtime registrar and call it with the configured callback unconditionally at module top level.",
            },
          });
      },
    };
  },
};
