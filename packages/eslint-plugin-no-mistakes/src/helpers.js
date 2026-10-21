"use strict";

const { logicalCall, memberPropertyName, propertyName } = require("./rules/module-mock-helpers");

const DEFAULT_SELECTOR_ATTRIBUTES = ["data-testid", "data-pw"];
const DEFAULT_NAMING_PATTERN = "^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$";
const INTERACTIVE_ELEMENTS = new Set(["button", "input", "select", "textarea"]);
const SELECTOR_METHODS = new Set([
  "$",
  "$$",
  "$eval",
  "$$eval",
  "check",
  "click",
  "dblclick",
  "dragAndDrop",
  "fill",
  "focus",
  "frameLocator",
  "getByTestId",
  "hover",
  "locator",
  "press",
  "selectOption",
  "setInputFiles",
  "tap",
  "textContent",
  "type",
  "uncheck",
  "waitForSelector",
]);

function options(context) {
  return context.options[0] || {};
}

function selectorAttributes(option) {
  return option.selectorAttributes || DEFAULT_SELECTOR_ATTRIBUTES;
}

function canonicalAttribute(option) {
  return option.canonicalAttribute || "data-pw";
}

function isSelectorAttribute(name, attrs) {
  return attrs.includes(name);
}

function attributeName(attribute) {
  if (!attribute || attribute.type !== "JSXAttribute") {
    return null;
  }
  if (attribute.name.type !== "JSXIdentifier") {
    return null;
  }
  return attribute.name.name;
}

function literalString(node) {
  if (node.type === "Literal" && typeof node.value === "string") {
    return node.value;
  }
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.raw).join("");
  }
  return null;
}

function staticTemplate(node) {
  if (node && node.type === "TemplateLiteral" && node.expressions.length > 0) {
    return node.quasis.some((quasi) => quasi.value.raw.length > 0);
  }
  return false;
}

function jsxAttributeExpression(attribute) {
  if (!attribute.value) {
    return null;
  }
  if (attribute.value.type === "JSXExpressionContainer") {
    return attribute.value.expression;
  }
  return attribute.value;
}

function selectorLiteral(attribute) {
  const expression = jsxAttributeExpression(attribute);
  if (!expression) {
    return null;
  }
  return literalString(expression);
}

function callMethodName(node) {
  if (node.callee.type === "MemberExpression" && !node.callee.computed) {
    return node.callee.property.name;
  }
  if (node.callee.type === "Identifier") {
    return node.callee.name;
  }
  return null;
}

function isSelectorCall(node) {
  const name = callMethodName(node);
  return Boolean(name && SELECTOR_METHODS.has(name));
}

function cssSelectorValues(source, attrs) {
  const values = [];
  for (const attr of attrs) {
    const escaped = attr.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(
      `\\[\\s*${escaped}\\s*([*^$]?=)\\s*(?:"([^"]*)"|'([^']*)'|([^\\s\\]]+))(?:[is]|\\s+[is])?\\s*\\]`,
      "g",
    );
    let match = regex.exec(source);
    while (match) {
      values.push({ attribute: attr, operator: match[1], value: match[2] ?? match[3] ?? match[4] });
      match = regex.exec(source);
    }
  }
  return values;
}

function isStringLiteralNode(node) {
  return literalString(node) !== null;
}

function isStaticString(node) {
  return Boolean(node && isStringLiteralNode(node));
}

function selectorValueNode(attribute) {
  const expression = jsxAttributeExpression(attribute);
  return expression && expression.type !== "JSXEmptyExpression" ? expression : null;
}

const LOCAL_BINDING_TYPES = new Set([
  "Variable",
  "Parameter",
  "CatchClause",
  "FunctionName",
  "ImportBinding",
  "ClassName",
]);

function isFetchShadowed(scope) {
  while (scope) {
    const get = scope.set?.get;
    const variable = typeof get === "function" ? get.call(scope.set, "fetch") : null;
    if (variable) {
      return variable.defs.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    }
    if (!scope.set || typeof get !== "function") {
      const fallback = scope.variables?.find((item) => item.name === "fetch");
      if (fallback) {
        return fallback.defs.some((def) => LOCAL_BINDING_TYPES.has(def.type));
      }
    }
    scope = scope.upper;
  }
  return false;
}

const FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);

function unboundRoot(node, context) {
  if (node?.type !== "Identifier" || !FETCH_ROOTS.has(node.name)) return false;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables?.find((item) => item.name === node.name);
    if (variable) return !variable.defs?.some((def) => LOCAL_BINDING_TYPES.has(def.type));
    scope = scope.upper;
  }
  return true;
}

function bindingVariable(node, context) {
  if (!context?.sourceCode?.getScope) return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables?.find((item) => item.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function fetchCallee(node, context, seen) {
  const visiting = seen || new Set();
  if (node.type === "MemberExpression") {
    if (memberPropertyName(node) !== "fetch") return false;
    const object = node.object.type === "ChainExpression" ? node.object.expression : node.object;
    return unboundRoot(object, context);
  }
  if (node.type === "Identifier") {
    if (node.name === "fetch" && !isFetchShadowed(context?.sourceCode?.getScope?.(node)))
      return true;
    const variable = bindingVariable(node, context);
    if (!variable || visiting.has(variable) || variable.defs.length !== 1) return false;
    visiting.add(variable);
    const def = variable.defs[0];
    if (def.type !== "Variable" || def.parent?.kind !== "const") return false;
    if (def.node.id.type === "Identifier") return fetchCallee(def.node.init, context, visiting);
    if (def.node.id.type !== "ObjectPattern") return false;
    for (const item of def.node.id.properties) {
      if (item.type === "Property") {
        const value = item.value.type === "AssignmentPattern" ? item.value.left : item.value;
        if (
          value.type === "Identifier" &&
          value.name === node.name &&
          propertyName(item.key) === "fetch"
        ) {
          return unboundRoot(def.node.init, context);
        }
      }
    }
    return false;
  }
  return false;
}

function fetchInvocation(node, context) {
  const invoked = logicalCall(node);
  if (!invoked?.callee || !fetchCallee(invoked.callee, context)) return null;
  return invoked.arguments;
}

function isFetchCall(node, context) {
  return fetchInvocation(node, context) !== null;
}

function rule(meta, create) {
  return { meta, create };
}

module.exports = {
  DEFAULT_NAMING_PATTERN,
  attributeName,
  callMethodName,
  canonicalAttribute,
  cssSelectorValues,
  INTERACTIVE_ELEMENTS,
  fetchInvocation,
  isFetchCall,
  isSelectorAttribute,
  isSelectorCall,
  isStringLiteralNode,
  isStaticString,
  literalString,
  options,
  rule,
  selectorAttributes,
  selectorLiteral,
  selectorValueNode,
  staticTemplate,
};
