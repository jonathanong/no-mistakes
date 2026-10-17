"use strict";

const RESTORATION_FAILURE = Symbol("planning artifact output restoration failure");

function outputRestorationFailure(restoreError, parked, outputPath, updateError, restored = false) {
  const recoveryPath = restored ? outputPath : parked;
  const error = new AggregateError(
    updateError ? [restoreError, updateError] : [restoreError],
    restored
      ? `planning artifact output directory failed validation after restoration at ${recoveryPath}`
      : `planning artifact output restoration failed; recover the parked directory at ${recoveryPath}`,
    { cause: restoreError },
  );
  error.code = restoreError.code;
  error[RESTORATION_FAILURE] = true;
  return error;
}

function isOutputRestorationFailure(error) {
  return error?.[RESTORATION_FAILURE] === true;
}

function preserveFailureReportingError(originalError, failureReportingError) {
  if (
    (typeof originalError === "object" && originalError !== null) ||
    typeof originalError === "function"
  ) {
    try {
      Object.defineProperty(originalError, "failureReportingError", {
        configurable: true,
        value: failureReportingError,
      });
      return originalError;
    } catch {
      // Fall through when a caller throws a frozen or otherwise non-extensible value.
    }
  }
  return new AggregateError(
    [originalError, failureReportingError],
    `planning artifact generation failed and failure reporting could not restore the output directory: ${failureReportingError.message}`,
    { cause: originalError },
  );
}

module.exports = {
  isOutputRestorationFailure,
  outputRestorationFailure,
  preserveFailureReportingError,
};
