"use strict";

const { attributeName, isSelectorAttribute, options, selectorAttributes } = require("./helpers");

function selectorAttributeVisitors(context, callback) {
  const attrs = selectorAttributes(options(context));
  return {
    JSXAttribute(node) {
      const name = attributeName(node);
      if (name && isSelectorAttribute(name, attrs)) {
        callback(node, name);
      }
    },
  };
}

module.exports = {
  selectorAttributeVisitors,
};
