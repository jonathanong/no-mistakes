"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");

function immutableDefinition(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) return null;
  if (variable.references.some((reference) => reference.isWrite() && !reference.init)) return null;
  return variable.defs[0];
}

function namespaceImport(node, context, options, seen = new Set()) {
  node = unwrapTs(node);
  if (node?.type !== "Identifier") return false;
  const def = immutableDefinition(node, context);
  if (!def || seen.has(def)) return false;
  seen.add(def);
  if (def.type === "ImportBinding") {
    return (
      def.node.type === "ImportNamespaceSpecifier" &&
      def.parent.importKind !== "type" &&
      (!options.importSpecifier || def.parent.source.value === options.importSpecifier)
    );
  }
  return (
    def.parent?.kind === "const" &&
    def.node.id.type === "Identifier" &&
    namespaceImport(def.node.init, context, options, seen)
  );
}

function functionBody(fn) {
  let body = unwrapTs(fn.body);
  if (body.type === "BlockStatement") {
    if (body.body.length !== 1 || body.body[0].type !== "ReturnStatement") return null;
    body = unwrapTs(body.body[0].argument);
  }
  if (body?.type === "AwaitExpression") body = unwrapTs(body.argument);
  return body?.type === "CallExpression" ? body : null;
}

// This resolver belongs to the lint invocation. It follows bindings, never names
// copied across scopes, and substitutes only one transparent local forwarder.
function executorCallResolver(bindings, context, options, propertyName, scopedExecutor) {
  function memberName(node) {
    const key = unwrapTs(node.property);
    if (
      node.computed &&
      key.type !== "Literal" &&
      !(key.type === "TemplateLiteral" && key.expressions.length === 0)
    )
      return null;
    return propertyName(node);
  }
  function target(expression, seen = new Set()) {
    const node = unwrapTs(expression);
    if (!node) return null;
    if (node.type === "SequenceExpression") return target(node.expressions.at(-1), seen);
    if (node.type === "MemberExpression") {
      const name = memberName(node);
      if (
        name &&
        options.executorNames.includes(name) &&
        namespaceImport(node.object, context, options)
      ) {
        return { name };
      }
      return bindings.queryMembers && name === "query" ? { name } : null;
    }
    if (node.type !== "Identifier") return null;
    if (!resolveVariable(node, context) && bindings.has(node.name)) return { name: node.name };
    const def = immutableDefinition(node, context);
    if (!def || seen.has(def)) return null;
    seen.add(def);
    if (def.type === "ImportBinding") {
      const imported = def.node.imported;
      const name = imported?.type === "Literal" ? imported.value : imported?.name;
      return def.node.type === "ImportSpecifier" &&
        def.parent.importKind !== "type" &&
        def.node.importKind !== "type" &&
        options.executorNames.includes(name) &&
        (!options.importSpecifier || def.parent.source.value === options.importSpecifier)
        ? { name: node.name }
        : null;
    }
    if (scopedExecutor(def, node)) return { name: node.name };
    if (def.node.type === "FunctionDeclaration") return { fn: def.node };
    if (def.parent?.kind !== "const") return null;
    const init = unwrapTs(def.node.init);
    if (!init) return null;
    if (def.node.id.type === "ObjectPattern") {
      const property = def.node.id.properties.find(
        (item) => item.type === "Property" && item.value === def.name,
      );
      if (!property || property.value.type !== "Identifier") return null;
      const name = memberName({
        type: "MemberExpression",
        computed: property.computed,
        property: property.key,
      });
      if (name && options.executorNames.includes(name) && namespaceImport(init, context, options))
        return { name };
      return bindings.queryMembers && name === "query" ? { name } : null;
    }
    if (def.node.id.type !== "Identifier") return null;
    if (init.type === "ArrowFunctionExpression" || init.type === "FunctionExpression")
      return { fn: init };
    return target(init, seen);
  }

  function resolve(call, forward = true) {
    const callee = unwrapTs(call?.callee);
    if (!callee) return null;
    let found = target(callee);
    let args = call.arguments;
    if (!found && callee.type === "MemberExpression") {
      const method = memberName(callee);
      if (method !== "call" && method !== "apply") return null;
      found = target(callee.object);
      if (!found || args.some((arg) => arg.type === "SpreadElement")) return null;
      if (method === "call") args = args.slice(1);
      else {
        const array = unwrapTs(args[1]);
        if (
          array?.type !== "ArrayExpression" ||
          array.elements.some((arg) => !arg || arg.type === "SpreadElement")
        )
          return null;
        args = array.elements;
      }
    }
    if (!found || args.some((arg) => arg.type === "SpreadElement")) return null;
    if (found.name) return { name: found.name, arguments: args };
    if (!forward || found.fn.params.some((param) => param.type !== "Identifier")) return null;
    const body = functionBody(found.fn);
    if (!body) return null;
    const inner = resolve(body, false);
    if (!inner) return null;
    const mapped = [];
    for (const argument of inner.arguments) {
      const value = unwrapTs(argument);
      if (value.type !== "Identifier") return null;
      const index = found.fn.params.findIndex(
        (param) => resolveVariable(param, context) === resolveVariable(value, context),
      );
      if (index < 0 || !args[index]) return null;
      mapped.push(args[index]);
    }
    return { name: inner.name, arguments: mapped };
  }
  // A callback passed as a value retains executor identity, not an invocation's
  // Function.prototype.call/apply receiver or argument forwarding semantics.
  resolve.reference = (expression) => Boolean(target(expression)?.name);
  return resolve;
}

module.exports = { executorCallResolver };
