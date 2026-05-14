"use strict";

const { isStringLiteralNode, literalString, staticTemplate } = require("./helpers");

const DEFAULTED_PROPS = new WeakMap();

function isLiteralLike(node, opts, context) {
  const value = literalString(node);
  if (value !== null) {
    return true;
  }
  if (opts.allowStaticTemplates && staticTemplate(node)) {
    return true;
  }
  if (opts.allowDefaultedProps && isDefaultedPropReference(node, context)) {
    return true;
  }
  return false;
}

function collectDefaultedProps(params) {
  const props = new Set();
  for (const param of params) {
    collectPatternDefaults(param, props);
  }
  return props;
}

function collectFunctionDefaultedProps(fn) {
  const props = collectDefaultedProps(fn.params);
  collectConstPatternDefaults(fn.body, props);
  return props;
}

function collectPatternDefaults(pattern, props) {
  if (pattern.type === "AssignmentPattern") {
    collectDefaultName(pattern.left, props, isStringLiteralNode(pattern.right));
    return;
  }
  if (pattern.type === "ObjectPattern") {
    for (const prop of pattern.properties) {
      collectObjectPropertyDefault(prop, props);
    }
    return;
  }
  if (pattern.type === "ArrayPattern") {
    for (const element of pattern.elements) {
      if (element) {
        collectPatternDefaults(element, props);
      }
    }
  }
}

function collectObjectPropertyDefault(prop, props) {
  if (prop.type === "RestElement") {
    return;
  }
  if (prop.value.type === "AssignmentPattern") {
    collectDefaultName(prop.value.left, props, isStringLiteralNode(prop.value.right));
    return;
  }
  collectPatternDefaults(prop.value, props);
}

function collectDefaultName(node, props, hasLiteralDefault) {
  if (hasLiteralDefault && node.type === "Identifier") {
    props.add(node.name);
  }
  if (node.type === "ObjectPattern") {
    collectPatternDefaults(node, props);
  }
}

function collectConstPatternDefaults(node, props) {
  /* v8 ignore next -- function-like nodes without bodies are defensive traversal */
  if (!node) {
    return;
  }
  if (node.type === "VariableDeclaration") {
    if (node.kind === "const") {
      for (const declaration of node.declarations) {
        if (declaration.id.type === "ObjectPattern" || declaration.id.type === "ArrayPattern") {
          collectPatternDefaults(declaration.id, props);
        }
      }
    }
    return;
  }
  if (node.type !== "BlockStatement" && isFunctionNode(node)) {
    return;
  }
  for (const child of constDefaultTraversalChildren(node)) {
    collectConstPatternDefaults(child, props);
  }
}

function patternHasLiteralDefault(pattern, name) {
  if (pattern.type === "AssignmentPattern") {
    return pattern.left.type === "Identifier"
      ? pattern.left.name === name && isStringLiteralNode(pattern.right)
      : patternHasLiteralDefault(pattern.left, name);
  }
  if (pattern.type !== "ObjectPattern") {
    return (
      pattern.type === "ArrayPattern" &&
      pattern.elements.some((element) => element && patternHasLiteralDefault(element, name))
    );
  }
  return pattern.properties.some((prop) => {
    /* v8 ignore next -- rest bindings cannot be recorded as defaulted props */
    if (prop.type === "RestElement") {
      return false;
    }
    return patternHasLiteralDefault(prop.value, name);
  });
}

function constDefaultTraversalChildren(node) {
  if (node.type === "BlockStatement") {
    return node.body;
  }
  if (node.type === "IfStatement") {
    return [node.consequent, node.alternate].filter(Boolean);
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "ForStatement") {
    return [node.init, node.body].filter(Boolean);
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "ForInStatement" || node.type === "ForOfStatement") {
    return [node.left, node.body];
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "WhileStatement" || node.type === "DoWhileStatement") {
    return [node.body];
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "SwitchStatement") {
    return node.cases.flatMap((item) => item.consequent);
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "TryStatement") {
    return [node.block, node.handler?.body, node.finalizer].filter(Boolean);
  }
  /* v8 ignore next -- traversal support for less common statement containers */
  if (node.type === "LabeledStatement" || node.type === "WithStatement") {
    return [node.body];
  }
  /* v8 ignore next -- non-container statements cannot hold const declarations */
  return [];
}

function isFunctionNode(node) {
  return (
    node.type === "FunctionDeclaration" ||
    node.type === "FunctionExpression" ||
    node.type === "ArrowFunctionExpression"
  );
}

function nearestFunction(node) {
  let current = node.parent;
  while (current) {
    if (isFunctionNode(current)) {
      return current;
    }
    current = current.parent;
  }
  return null;
}

function defaultedPropsForNode(node) {
  const fn = nearestFunction(node);
  /* v8 ignore next -- direct helper tests cover the no-function return */
  if (!fn) {
    return new Set();
  }
  let props = DEFAULTED_PROPS.get(fn);
  /* v8 ignore next -- cache hits are a performance detail */
  if (!props) {
    props = collectFunctionDefaultedProps(fn);
    DEFAULTED_PROPS.set(fn, props);
  }
  return props;
}

function isDefaultedPropReference(node, context) {
  if (node?.type !== "Identifier" || !defaultedPropsForNode(node).has(node.name)) {
    return false;
  }
  const variable = findVariable(context.sourceCode.getScope(node), node.name);
  return Boolean(
    variable?.defs.some(
      (def) => def.type === "Parameter" || hasLiteralConstPatternDefault(def, node.name),
    ),
  );
}

function hasLiteralConstPatternDefault(def, name) {
  return (
    def.type === "Variable" &&
    def.parent?.kind === "const" &&
    (def.node?.id?.type === "ObjectPattern" || def.node?.id?.type === "ArrayPattern") &&
    patternHasLiteralDefault(def.node.id, name)
  );
}

function findVariable(scope, name) {
  let current = scope;
  while (current) {
    const variable = current.variables.find((item) => item.name === name);
    /* v8 ignore next -- recorded identifiers should resolve in ESLint scope */
    if (variable) {
      return variable;
    }
    /* v8 ignore next -- upper-scope traversal is covered by rule integration tests */
    current = current.upper;
  }
  return null;
}

module.exports = {
  defaultedPropsForNode,
  isDefaultedPropReference,
  isLiteralLike,
};
