"use strict";

const api = require("./no-object-mutation-runtime");
const { assignPattern, bindPattern } = require("./no-object-mutation-bind");
const { elementTag, unknown, child } = require("./no-object-mutation-values");

function cellRank(tag) {
  if (tag.type === "union") {
    let rank = 0;
    for (const member of tag.members) {
      rank = Math.max(rank, cellRank(member));
    }
    return rank;
  }
  if (tag.type === "fresh") {
    return tag.escaped ? 1 : 0;
  }
  if (tag.type === "shared" || tag.type === "this") {
    return 1;
  }
  return 0;
}

function carriedCells(scope) {
  const cells = [];
  for (let cursor = scope; cursor; cursor = cursor.outer) {
    for (const cell of cursor.cells.values()) {
      cells.push(cell);
    }
  }
  return cells;
}

// The write uses the previous iteration's value, which the first pass has
// not stored yet. Rank rises only when a carried cell becomes shared or
// escaped, so a purely local loop does not walk again. A fresh escape inside
// the body gets one extra pass; new objects must not force a pass each time.
function walkLoop(state, walkBody, scope) {
  const seen = new Map();
  let epochRewalk = false;
  for (let pass = 0; pass < 32; pass += 1) {
    const beforeEpoch = state.escapeEpoch;
    walkBody();
    let ranks = false;
    for (const cell of carriedCells(scope)) {
      const rank = cellRank(cell.tag);
      if (rank > (seen.get(cell) || 0)) {
        seen.set(cell, rank);
        ranks = true;
      }
    }
    if (ranks) {
      epochRewalk = false;
      continue;
    }
    if (state.escapeEpoch !== beforeEpoch && !epochRewalk) {
      epochRewalk = true;
      continue;
    }
    return;
  }
}

function visitLoop(state, node, scope) {
  if (node.type === "WhileStatement" || node.type === "DoWhileStatement") {
    walkLoop(
      state,
      () => {
        api.visit(state, node.test, scope, "read");
        api.visit(state, node.body, scope, "read");
      },
      scope,
    );
    return unknown();
  }
  if (node.type === "ForStatement") {
    if (node.init) {
      api.visit(state, node.init, scope, "read");
    }
    walkLoop(
      state,
      () => {
        if (node.test) {
          api.visit(state, node.test, scope, "read");
        }
        api.visit(state, node.body, scope, "read");
        if (node.update) {
          api.visit(state, node.update, scope, "read");
        }
      },
      scope,
    );
    return unknown();
  }
  walkLoop(
    state,
    () => {
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
    },
    scope,
  );
  return unknown();
}

module.exports = { visitLoop };
