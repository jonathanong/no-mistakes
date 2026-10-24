"use strict";

const { attributeName, isStaticString, literalString, selectorValueNode } = require("../helpers");
const { unwrapExpression } = require("./async-ast");

// A spread invalidates earlier props; a later explicit prop establishes its value again.
function effectiveAttribute(opening, name) {
  let found = null;
  for (const attribute of opening.attributes) {
    if (attribute.type === "JSXSpreadAttribute") found = null;
    else if (attributeName(attribute) === name) found = attribute;
  }
  return found;
}

function hasSlotProp(opening, names) {
  return names.some((name) => {
    const attribute = effectiveAttribute(opening, name);
    if (!attribute) return false;
    if (!attribute.value) return true;
    const value = unwrapExpression(selectorValueNode(attribute));
    if (value?.type === "Literal") return Boolean(value.value);
    return isStaticString(value) && Boolean(literalString(value));
  });
}

function literalSelectors(opening, names) {
  return names.filter((name) => {
    const attribute = effectiveAttribute(opening, name);
    return attribute && isStaticString(unwrapExpression(selectorValueNode(attribute)));
  });
}

function singleElementChild(element) {
  const children = element.children.filter((child) => {
    if (child.type === "JSXExpressionContainer" && child.expression.type === "JSXEmptyExpression")
      return false;
    if (child.type !== "JSXText") return true;
    // JSX drops indentation around newlines. Same-line spaces and nonbreaking
    // spaces are actual text siblings, so a Slot cannot merge onto one element.
    return child.value !== "" && !(/^[ \t\r\n]*$/.test(child.value) && /[\r\n]/.test(child.value));
  });
  return children.length === 1 && children[0].type === "JSXElement" ? children[0] : null;
}

function childKeepsSelector(opening, names) {
  return names.some(
    (name) =>
      !opening.attributes.some(
        (attribute) => attribute.type === "JSXSpreadAttribute" || attributeName(attribute) === name,
      ),
  );
}

module.exports = { childKeepsSelector, hasSlotProp, literalSelectors, singleElementChild };
