"use strict";

const api = require("./no-object-mutation-runtime");
const { assignPattern, bindPattern } = require("./no-object-mutation-bind");
const { elementTag, unknown, child } = require("./no-object-mutation-values");

function walkLoop(state, walkBody) {
  const before = state.escapeEpoch;
  walkBody();
  if (state.escapeEpoch !== before) {
    walkBody();
  }
}

function visitLoop(state, node, scope) {
  if (node.type === "WhileStatement" || node.type === "DoWhileStatement") {
    walkLoop(state, () => {
      api.visit(state, node.test, scope, "read");
      api.visit(state, node.body, scope, "read");
    });
    return unknown();
  }
  if (node.type === "ForStatement") {
    if (node.init) {
      api.visit(state, node.init, scope, "read");
    }
    walkLoop(state, () => {
      if (node.test) {
        api.visit(state, node.test, scope, "read");
      }
      api.visit(state, node.body, scope, "read");
      if (node.update) {
        api.visit(state, node.update, scope, "read");
      }
    });
    return unknown();
  }
  walkLoop(state, () => {
    const loop = child(scope);
    const right = api.visit(state, node.right, loop, "read");
    const bound = node.type === "ForOfStatement" ? elementTag(right) : unknown();
    if (node.left.type === "VariableDeclaration") {
      const kind = node.left.kind === "var" ? "var" : "local";
      for (const declarator of node.left.declarations) {
        bindPattern(state, declarator.id, bound, loop, kind);
      }
    } else {
      assignPattern(state, node.left, bound, loop);
    }
    api.visit(state, node.body, loop, "read");
  });
  return unknown();
}

module.exports = { visitLoop };
