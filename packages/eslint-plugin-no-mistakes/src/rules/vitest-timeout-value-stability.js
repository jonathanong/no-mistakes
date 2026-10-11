"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapTransparentParent } = require("./async-ast");
const { configCall, overrideCall, invocationArguments } = require("./vitest-timeout-bindings");

// Borrow ESLint's existing variable references; do not build another binding graph.
function createValueStability(context, primitiveMemberRead) {
  // Values, copies, and callbacks need independent consumer proofs and cycle guards.
  const memo = { value: new WeakMap(), shallow: new WeakMap(), callback: new WeakMap() };
  const active = { value: new Set(), shallow: new Set(), callback: new Set() };
  function stable(variable, mode = "value") {
    const cache = memo[mode],
      visiting = active[mode];
    if (!variable || visiting.has(variable)) return false;
    if (cache.has(variable)) return cache.get(variable);
    visiting.add(variable);
    const result =
      variable.defs.every(
        (def) =>
          def.type === "Variable" &&
          def.parent.kind === "const" &&
          def.parent.parent?.type !== "ExportNamedDeclaration" &&
          def.parent.parent?.type !== "ExportDefaultDeclaration",
      ) &&
      variable.references.every(
        (reference) =>
          reference.init ||
          // TypeScript's erased references cannot expose a runtime value.
          typeOnlyUse(reference.identifier) ||
          (!reference.isWrite() && safeUse(reference.identifier, mode)),
      );
    visiting.delete(variable);
    cache.set(variable, result);
    return result;
  }
  function typeOnlyUse(node) {
    while (node.parent?.type === "TSQualifiedName" && node.parent.left === node) node = node.parent;
    return node.parent?.type === "TSTypeQuery" && node.parent.exprName === node;
  }
  function configCallback(node) {
    return (
      (node?.type === "ArrowFunctionExpression" || node?.type === "FunctionExpression") &&
      !node.async &&
      !node.generator &&
      node.params.every((param) => param.type === "Identifier") &&
      safeUse(node, "callback")
    );
  }
  function confinedConfigResult(call) {
    const node = unwrapTransparentParent(call);
    const parent = node.parent;
    if (parent?.type === "ExpressionStatement") return true;
    // mergeConfig creates an object rather than retaining the callback identity.
    if (
      parent?.type === "CallExpression" &&
      configCall(parent, context) === "mergeConfig" &&
      (parent.arguments[0] === node || parent.arguments[1] === node)
    )
      return true;
    return safeUse(node, "callback");
  }
  function safeUse(identifier, mode = "value") {
    let node = unwrapTransparentParent(identifier);
    if (mode === "callback") {
      const parent = node.parent;
      if (parent?.type === "VariableDeclarator" && parent.init === node)
        return (
          parent.id.type === "Identifier" && stable(resolveVariable(parent.id, context), "callback")
        );
      // Test callbacks and mergeConfig object operands do not prove a config callback consumer.
      return (
        parent?.type === "CallExpression" &&
        parent.arguments[0] === node &&
        ["defineConfig", "defineProject"].includes(configCall(parent, context)) &&
        confinedConfigResult(parent)
      );
    }
    let memberDepth = 0;
    while (node.parent?.type === "MemberExpression" && node.parent.object === node) {
      node = unwrapTransparentParent(node.parent);
      memberDepth++;
    }
    const memberRead = node.type === "MemberExpression" && primitiveMemberRead(node);
    let parent = node.parent;
    if (
      (parent?.type === "AssignmentExpression" && parent.left === node) ||
      (parent?.type === "UpdateExpression" && parent.argument === node) ||
      (parent?.type === "UnaryExpression" && parent.operator === "delete")
    )
      // Replacing a copy's own property cannot mutate the source. Its children
      // are still shared, so writes through a second member remain unsafe.
      return mode === "shallow" && memberDepth === 1;
    // A proven scalar read cannot expose the receiver, but writes above still
    // invalidate it. Object-valued and unresolved properties remain conservative.
    if (memberRead) return true;
    if (memberDepth) mode = "value";
    // Container copies can preserve nested references. Follow their existing binding too.
    while (
      parent &&
      ((parent.type === "Property" && parent.value === node) ||
        parent.type === "ObjectExpression" ||
        parent.type === "ArrayExpression" ||
        parent.type === "SpreadElement")
    ) {
      if (
        parent.type === "SpreadElement" &&
        (parent.parent.type === "ObjectExpression" || parent.parent.type === "ArrayExpression")
      )
        mode = "shallow";
      node = parent;
      parent = node.parent;
    }
    if (parent?.type === "VariableDeclarator" && parent.init === node)
      return parent.id.type === "Identifier" && stable(resolveVariable(parent.id, context), mode);
    if (parent?.type === "CallExpression") {
      if (parent.callee === node) return false;
      // Public config/registration consumers are admitted; unknown helpers can mutate objects.
      const config = configCall(parent, context);
      if (config)
        return (
          parent.arguments.indexOf(node) === 0 ||
          (config === "mergeConfig" && parent.arguments.indexOf(node) === 1)
        );
      const args = invocationArguments(parent, context);
      const index = args.indexOf(node);
      const kind = overrideCall(parent, context);
      return (
        (kind === "runtime" && index === 0) ||
        (kind === "hook" && index === 1) ||
        (kind === "test" && (index === 1 || index === 2))
      );
    }
    if (parent?.type === "ReturnStatement") {
      const body = parent.parent;
      return (
        body?.type === "BlockStatement" && body.body.length === 1 && configCallback(body.parent)
      );
    }
    if (parent?.type === "ArrowFunctionExpression" && parent.body === node)
      return configCallback(parent);
    // Exports and unproved contexts (await/yield, tagged templates, computed
    // keys, and other consumers) can expose the object. No default safe escape.
    return false;
  }
  return stable;
}
module.exports = { createValueStability };
