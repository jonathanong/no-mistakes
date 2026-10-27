"use strict";
const { isReference, write, lookup, scopeViews } = require("./async-source-ts-references");

// Build only lexical definitions/references required by the existing proof engine.
// Two walks preserve hoisting, shadowing, captures, and writes before declarations.
function sourceScopes(ast, visitorKeys) {
  const scopes = [];
  const owners = new WeakMap();
  const typePositions = new WeakMap();
  const bindings = new WeakSet();
  function scope(upper, type) {
    const result = { upper, type, variables: [], references: [], through: [] };
    scopes.push(result);
    return result;
  }
  const globalScope = scope(null, "global");
  function each(node, visit) {
    for (const key of visitorKeys[node.type] || []) {
      const value = node[key];
      if (Array.isArray(value)) value.forEach((child) => child && visit(child, key));
      else if (value) visit(value, key);
    }
  }
  function define(pattern, current, type, node, parent) {
    if (!pattern) return;
    if (pattern.type === "Identifier") {
      bindings.add(pattern);
      let variable = current.variables.find((item) => item.name === pattern.name);
      if (!variable) {
        variable = { name: pattern.name, defs: [], references: [], identifiers: [] };
        current.variables.push(variable);
      }
      variable.identifiers.push(pattern);
      variable.defs.push({ type, name: pattern, node, parent });
    } else if (pattern.type === "AssignmentPattern")
      define(pattern.left, current, type, node, parent);
    else if (pattern.type === "RestElement") define(pattern.argument, current, type, node, parent);
    else if (pattern.type === "ObjectPattern")
      pattern.properties.forEach((item) =>
        define(item.type === "Property" ? item.value : item, current, type, node, parent),
      );
    else if (pattern.type === "ArrayPattern")
      pattern.elements.forEach((item) => define(item, current, type, node, parent));
  }
  const functions = new Set([
    "FunctionDeclaration",
    "TSDeclareFunction",
    "FunctionExpression",
    "ArrowFunctionExpression",
    "TSFunctionType",
    "TSMethodSignature",
    "TSIndexSignature",
  ]);
  function declarations(node, current) {
    if (["FunctionDeclaration", "TSDeclareFunction"].includes(node.type))
      define(node.id, current, "FunctionName", node, node.parent);
    if (["TSTypeAliasDeclaration", "TSInterfaceDeclaration"].includes(node.type)) {
      define(node.id, current, "Type", node, node.parent);
      current = scope(current, "type");
    }
    if (["TSConditionalType", "TSMappedType"].includes(node.type)) current = scope(current, "type");
    if (functions.has(node.type)) {
      current = scope(current, "function");
      if (node.type === "FunctionExpression") define(node.id, current, "FunctionName", node, node);
      (node.params || node.parameters).forEach((param) =>
        define(param, current, "Parameter", node, node),
      );
    } else if (
      [
        "BlockStatement",
        "ForStatement",
        "ForOfStatement",
        "ForInStatement",
        "CatchClause",
      ].includes(node.type)
    )
      // Body vars cannot shadow references in default parameter initializers.
      current = scope(
        current,
        node.type === "BlockStatement" &&
          functions.has(node.parent.type) &&
          node.parent.params.some((param) => param.type !== "Identifier")
          ? "function-body"
          : "block",
      );
    owners.set(node, current);
    if (node.type === "VariableDeclarator") {
      let target = current;
      if (node.parent.kind === "var")
        while (target.upper && !["function", "function-body"].includes(target.type))
          target = target.upper;
      define(node.id, target, "Variable", node, node.parent);
    }
    if (node.type.startsWith("Import") && node.local)
      define(node.local, current, "ImportBinding", node, node.parent);
    if (node.type === "CatchClause") define(node.param, current, "CatchClause", node, node);
    if (node.type === "TSTypeParameter") define(node.name, current, "Type", node, node.parent);
    each(node, (child) => declarations(child, current));
  }
  declarations(ast, globalScope);
  function add(identifier, init, writing, typeOnly) {
    const current = owners.get(identifier);
    const resolved = lookup(current, identifier.name, typeOnly);
    const reference = {
      identifier,
      init,
      from: current,
      resolved,
      isWrite: () => writing,
      isTypeReference: typeOnly,
      isValueReference: !typeOnly,
    };
    current.references.push(reference);
    if (resolved) resolved.references.push(reference);
    else globalScope.through.push(reference);
  }
  function initializations(pattern) {
    if (!pattern) return;
    if (pattern.type === "Identifier") add(pattern, true, true, false);
    else if (pattern.type === "AssignmentPattern") initializations(pattern.left);
    else if (pattern.type === "RestElement") initializations(pattern.argument);
    else if (pattern.type === "ObjectPattern")
      pattern.properties.forEach((item) =>
        initializations(item.type === "Property" ? item.value : item),
      );
    else if (pattern.type === "ArrayPattern") pattern.elements.forEach(initializations);
  }
  function references(node, typeOnly = false) {
    typePositions.set(node, typeOnly);
    if (node.type === "Identifier" && isReference(node, bindings))
      add(node, false, write(node), typeOnly);
    if (
      node.type === "VariableDeclarator" &&
      (node.init || ["ForOfStatement", "ForInStatement"].includes(node.parent.parent.type))
    )
      initializations(node.id);
    each(node, (child, key) => {
      let typed = typeOnly;
      if (["typeAnnotation", "returnType", "typeArguments", "typeParameters"].includes(key))
        typed = true;
      if (node.type === "TSTypeQuery" && key === "exprName") typed = false;
      if (["TSTypeAliasDeclaration", "TSInterfaceBody", "TSInterfaceHeritage"].includes(node.type))
        typed = true;
      references(child, typed);
    });
  }
  references(ast);
  return {
    scopeManager: { scopes, globalScope, acquire: scopeViews(owners, typePositions) },
  };
}

module.exports = { sourceScopes };
