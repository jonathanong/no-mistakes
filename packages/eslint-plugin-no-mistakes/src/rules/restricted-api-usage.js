"use strict";

const { literalString } = require("./module-mock-helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { isLocalRequire, memberPropertyName, resolveVariable } = require("./async-target-bindings");

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
    const hasCallTargets = targets.some((target) => target.call);
    function report(node, target, name, rewrite = "") {
      if (suppressed(node)) return;
      context.report({
        node,
        messageId: "restrictedApi",
        data: {
          message:
            (target.message || `Restricted API '${name}'. Use an allowed API instead.`) + rewrite,
        },
      });
    }
    function recordImports(program) {
      for (const node of program.body) {
        if (node.type !== "ImportDeclaration" || node.importKind === "type") continue;
        for (const specifier of node.specifiers) {
          if (specifier.importKind === "type") continue;
          // A default import is the default value, not a module namespace.
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
    function namespaceBinding(expression) {
      const object = unwrapExpression(expression);
      if (!bindings.size || object.type !== "Identifier") return null;
      const binding = bindings.get(resolveVariable(object, context));
      return binding?.name === null ? binding : null;
    }
    function matches(target, binding, name) {
      return (
        binding && name !== null && target.source === binding.source && target.imported?.test(name)
      );
    }
    function member(node) {
      const binding = namespaceBinding(node.object);
      const name = memberPropertyName(node);
      for (const target of targets) if (matches(target, binding, name)) report(node, target, name);
    }
    function destructure(node) {
      const parent = node.parent;
      const value =
        parent.type === "VariableDeclarator"
          ? parent.init
          : parent.type === "AssignmentExpression"
            ? parent.right
            : null;
      if (!value) return;
      const binding = namespaceBinding(value);
      for (const property of node.properties) {
        if (property.type === "RestElement" && binding) sourceLoad(property, binding.source);
        if (property.type !== "Property") continue;
        const name = memberPropertyName({ property: property.key, computed: property.computed });
        for (const target of targets)
          if (matches(target, binding, name)) report(property, target, name);
      }
    }
    function reexport(node) {
      if (!node.source || node.exportKind === "type") return;
      const binding = { source: node.source.value };
      for (const target of targets) {
        if (node.type === "ExportAllDeclaration") {
          if (target.source === binding.source) report(node, target, binding.source);
          continue;
        }
        for (const specifier of node.specifiers) {
          if (specifier.exportKind === "type") continue;
          const name = specifier.local.name ?? specifier.local.value;
          if (matches(target, binding, name)) report(specifier, target, name);
        }
      }
    }
    function sourceLoad(node, source) {
      for (const target of targets) {
        if (target.source === source) {
          report(
            node,
            target,
            source,
            " Use a static named import so the imported API can be checked.",
          );
        }
      }
    }
    function dynamicSource(node, source) {
      const value = literalString(unwrapExpression(source));
      if (value !== null) sourceLoad(node, value);
    }
    function namespaceAlias(node) {
      const value = node.type === "VariableDeclarator" ? node.init : node.right;
      const destination = node.type === "VariableDeclarator" ? node.id : node.left;
      if (!value || destination.type !== "Identifier") return;
      const binding = namespaceBinding(value);
      if (binding) sourceLoad(value, binding.source);
    }
    function invocation(node) {
      const callee = unwrapExpression(node.callee);
      if (
        callee.type === "Identifier" &&
        callee.name === "require" &&
        !isLocalRequire(callee, context)
      ) {
        dynamicSource(node, node.arguments[0]);
      }
      if (!bindings.size && !hasCallTargets) return;
      const isMember = callee.type === "MemberExpression";
      const name =
        callee.type === "Identifier" ? callee.name : isMember ? memberPropertyName(callee) : null;
      const binding =
        callee.type === "Identifier" && bindings.size
          ? bindings.get(resolveVariable(callee, context))
          : null;
      const namespace = isMember ? namespaceBinding(callee.object) : null;
      for (const target of targets) {
        // Namespace accesses report at the member itself, including calls.
        if (matches(target, namespace, name)) continue;
        if (
          (name !== null && target.call?.test(name)) ||
          matches(target, binding, binding?.name ?? null)
        ) {
          report(node, target, binding?.name ?? name);
        }
      }
    }
    return {
      Program: recordImports,
      CallExpression: invocation,
      MemberExpression: member,
      ObjectPattern: destructure,
      VariableDeclarator: namespaceAlias,
      AssignmentExpression: namespaceAlias,
      ExportNamedDeclaration: reexport,
      ExportAllDeclaration: reexport,
      ImportExpression(node) {
        dynamicSource(node, node.source);
      },
    };
  },
};
