"use strict";

const api = require("./no-object-mutation-runtime");

function escapeResult(state, tag, scope, role, release = api.releaseEscaping) {
  if (role === "return") {
    scope.functionEnv.returned.push({ tag, release });
  } else if (role === "escape") {
    release(state, tag);
  }
}

module.exports = { escapeResult };
