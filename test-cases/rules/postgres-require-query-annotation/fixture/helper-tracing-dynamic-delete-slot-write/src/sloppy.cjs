const database = require("@app/db");

function dynamicDeleteThenReplace(statement, key) {
  const slots = arguments;
  delete slots[key];
  slots[0] = "/* replacement slot */ SELECT 1";
  database.query(statement); // finding:possible-disconnection-keeps-old-formal
}

function noDynamicDelete(statement) {
  const slots = arguments;
  slots[0] = "/* definite mapped replacement */ SELECT 1";
  database.query(statement); // known:definite-mapped-replacement
}

function strictDynamicDeleteThenReplace(statement, key) {
  "use strict";
  const slots = arguments;
  delete slots[key];
  slots[0] = "/* strict replacement */ SELECT 1";
  database.query(statement); // finding:strict-delete-does-not-map
}

function conditionalDeleteThenReplaceTwice(statement, remove) {
  const slots = arguments;
  const deletion = remove ? delete slots[0] : false; // Keep this in the supported conditional-expression join.
  slots[0] = "/* first conditional replacement */ SELECT 1";
  slots[0] = "/* second conditional replacement */ SELECT 1";
  database.query(statement); // finding:conditional-delete-keeps-original-formal
}

function opaqueThenDynamicDeleteAndReplace(statement, key) {
  const slots = arguments;
  const escaped = opaque(arguments);
  delete slots[key];
  slots[0] = "/* post-escape replacement */ SELECT 1";
  database.query(statement); // finding:post-escape-dynamic-delete-keeps-formal
}

dynamicDeleteThenReplace("SELECT 1", dynamicKey);
noDynamicDelete("SELECT 1");
strictDynamicDeleteThenReplace("SELECT 1", dynamicKey);
conditionalDeleteThenReplaceTwice("SELECT 1", maybeRemove);
opaqueThenDynamicDeleteAndReplace("SELECT 1", dynamicKey);
