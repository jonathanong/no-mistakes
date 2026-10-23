"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");

const targetSchema = {
  type: "object",
  properties: {
    source: { type: "string", minLength: 1 },
    importedNamePattern: { type: "string", minLength: 1 },
    callNamePattern: { type: "string", minLength: 1 },
    message: { type: "string", minLength: 1 },
  },
  anyOf: [{ required: ["source", "importedNamePattern"] }, { required: ["callNamePattern"] }],
  dependencies: { importedNamePattern: ["source"], source: ["importedNamePattern"] },
  additionalProperties: false,
};

module.exports = {
  meta: {
    type: "problem",
    docs: { description: "restrict configured API imports and calls", recommended: false },
    schema: [
      {
        type: "object",
        properties: { targets: { type: "array", items: targetSchema, minItems: 1 } },
        additionalProperties: false,
      },
    ],
    messages: { restrictedApi: "{{message}}" },
  },
  create(context) {
    const targets = (context.options[0]?.targets || []).map((target) => ({
      ...target,
      imported: target.importedNamePattern ? new RegExp(target.importedNamePattern) : null,
      call: target.callNamePattern ? new RegExp(target.callNamePattern) : null,
    }));
    if (!targets.length) return {};
    const suppressed = ruleSuppression(context, "restricted-api-usage");
    const bindings = new Map();
    function report(node, target, name) {
      if (suppressed(node)) return;
      context.report({
        node,
        messageId: "restrictedApi",
        data: {
          message: target.message || `Restricted API '${name}'. Use an allowed API instead.`,
        },
      });
    }
    function recordImports(program) {
      for (const node of program.body) {
        if (node.type !== "ImportDeclaration" || node.importKind === "type") continue;
        for (const specifier of node.specifiers) {
          if (specifier.importKind === "type") continue;
          const name =
            specifier.type === "ImportSpecifier"
              ? (specifier.imported.name ?? specifier.imported.value)
              : specifier.type === "ImportDefaultSpecifier"
                ? "default"
                : null;
          bindings.set(resolveVariable(specifier.local, context), {
            source: node.source.value,
            name,
          });
          if (name === null) continue;
          for (const target of targets) {
            if (target.source === node.source.value && target.imported?.test(name)) {
              report(specifier, target, name);
            }
          }
        }
      }
    }
    function invocation(node) {
      const callee = unwrapExpression(node.callee);
      const isMember = callee.type === "MemberExpression";
      const name =
        callee.type === "Identifier" ? callee.name : isMember ? memberPropertyName(callee) : null;
      let binding = null;
      if (callee.type === "Identifier") binding = bindings.get(resolveVariable(callee, context));
      else if (isMember) {
        const object = unwrapExpression(callee.object);
        if (object.type === "Identifier") {
          const namespace = bindings.get(resolveVariable(object, context));
          if (namespace?.name === null) binding = { source: namespace.source, name };
        }
      }
      for (const target of targets) {
        if (
          (name !== null && target.call?.test(name)) ||
          (binding &&
            binding.name !== null &&
            target.source === binding.source &&
            target.imported?.test(binding.name))
        ) {
          report(node, target, binding?.name ?? name);
        }
      }
    }
    return { Program: recordImports, CallExpression: invocation };
  },
};
