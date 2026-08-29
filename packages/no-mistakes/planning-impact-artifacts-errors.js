"use strict";

const RESTORATION_FAILURE = Symbol("planning artifact output restoration failure");

function outputRestorationFailure(restoreError, parked, updateError) {
  const error = new AggregateError(
    updateError ? [restoreError, updateError] : [restoreError],
    `planning artifact output restoration failed; recover the parked directory at ${parked}`,
    { cause: restoreError },
  );
  error.code = restoreError.code;
  error[RESTORATION_FAILURE] = true;
  return error;
}

function isOutputRestorationFailure(error) {
  return error?.[RESTORATION_FAILURE] === true;
}

module.exports = { isOutputRestorationFailure, outputRestorationFailure };
