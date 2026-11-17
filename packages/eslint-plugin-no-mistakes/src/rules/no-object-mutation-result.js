"use strict";

const api = require("./no-object-mutation-runtime");

function escapeResult(state, tag, scope, role, release = api.releaseEscaping) {
  if (role === "return") {
    const changes = new Map();
    state.returnEscapes ||= [];
    state.returnEscapes.push(changes);
    release(state, tag);
    state.returnEscapes.pop();
    for (const [value, { key, previous }] of changes) {
      scope.functionEnv.returned.push({ tag: value, key, value: value[key] });
      value[key] = previous;
    }
  } else if (role === "escape") {
    release(state, tag);
  }
}

module.exports = { escapeResult };
