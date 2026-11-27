"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { resultDiscarded } = require("./restricted-api-usage-discard");
const { targetSchema } = require("./restricted-api-usage-schema");
const {
  findingMessage,
  mergeVisitors,
  rememberFinding,
  strictVisitors,
} = require("./restricted-api-usage-strict");
const { isLocalRequire, memberPropertyName, resolveVariable } = require("./async-target-bindings");
const {
  matches,
  namespaceBinding: resolveNamespace,
  namespaceVisitors,
} = require("./restricted-api-usage-namespace");

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
    messages: {
      restrictedApi: "{{message}}",
      unanalyzable:
        "Cannot tell whether this member of {{name}} is restricted. Write a static member so the restricted API can be checked.",
    },
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
    const reported = new WeakMap();
    const hasCallTargets = targets.some((target) => target.call);
    function report(node, target, name, rewrite = "") {
      if (suppressed(node)) return;
      const message = findingMessage(target, name, rewrite);
      if (!rememberFinding(reported, node, message)) return;
      context.report({ node, messageId: "restrictedApi", data: { message } });
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
            if (
              (target.source === undefined || target.source === node.source.value) &&
              target.imported?.test(name)
            ) {
              report(specifier, target, name);
            }
          }
        }
      }
    }
    const resolved = namespaceVisitors({ context, targets, report, bindings, suppressed });
    function namespaceBinding(expression) {
      return resolveNamespace(expression, context, bindings);
    }
    function invocation(node) {
      const callee = unwrapExpression(node.callee);
      if (
        callee.type === "Identifier" &&
        callee.name === "require" &&
        !isLocalRequire(callee, context)
      ) {
        resolved.dynamicSource(node, node.arguments[0]);
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
        const lexical =
          name !== null &&
          target.call?.test(name) &&
          (!target.discardedResultOnly || resultDiscarded(node));
        if (lexical || matches(target, binding, binding?.name ?? null)) {
          report(node, target, binding?.name ?? name);
        }
      }
    }
    const strict = targets.some((target) =>
      Object.keys(target).some((key) =>
        /^(reference|declaration|property|destructure|construct)Name/.test(key),
      ),
    );
    const base = {
      Program: recordImports,
      CallExpression: invocation,
      ...resolved.visitors,
      ImportExpression(node) {
        resolved.dynamicSource(node, node.source);
      },
    };
    return strict ? mergeVisitors(base, strictVisitors(targets, report)) : base;
  },
};
