import { mkdtempSync } from "node:fs";

const arrowHelper = (arrowCallback) => arrowCallback(mkdtempSync("arrow-"));
const expressionHelper = function (expressionCallback) {
  expressionCallback(mkdtempSync("expression-"));
};
function declaredHelper(declaredCallback) {
  declaredCallback(mkdtempSync("declared-"));
}
const scalarHelper = 42;
let absentHelper;
function destructuredHelper({ callback }) {}
function unusedHelper(unusedCallback) {}
function repositoryHelper(repositoryCallback) {
  repositoryCallback("repository");
}
function aliasHelper(aliasCallback) {
  const directAlias = aliasCallback;
  directAlias(mkdtempSync("alias-"));
}
function ignoredHelper(ignoredCallback) {
  type CallbackType = typeof ignoredCallback;
  const { nestedAlias } = ignoredCallback;
  ignoredCallback();
  ignoredCallback(...unknownArguments);
  consume(ignoredCallback);
}
function cycleHelper(cycleCallback) {
  var cycleCallback = cycleCallback;
}
function depthHelper(depthCallback) {
  const alias1 = depthCallback;
  const alias2 = alias1;
  const alias3 = alias2;
  const alias4 = alias3;
  const alias5 = alias4;
  const alias6 = alias5;
  const alias7 = alias6;
  const alias8 = alias7;
  alias8("repository");
}
function shortHelper(shortCallback) {
  const shortAlias1 = shortCallback;
  const shortAlias2 = shortAlias1;
  shortAlias2("repository");
}
