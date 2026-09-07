"use strict";

const {
  abruptCompletionReachesMatcher,
  alwaysExits,
  caughtThrowCanContinue,
  contains,
  continueSkipsMatcher,
} = require("./test-no-delayed-rejects-abrupt");

function finalizerPreventsReach(parent, child, matcher) {
  if (
    parent?.type !== "TryStatement" ||
    child === parent.finalizer ||
    !parent.finalizer ||
    contains(parent, matcher)
  ) {
    return false;
  }
  if (continueSkipsMatcher(parent.finalizer, matcher)) return true;
  return (
    alwaysExits(parent.finalizer) &&
    !contains(parent.finalizer, matcher) &&
    !abruptCompletionReachesMatcher(parent.finalizer, matcher) &&
    !caughtThrowCanContinue(parent.finalizer, matcher)
  );
}

module.exports = { finalizerPreventsReach };
