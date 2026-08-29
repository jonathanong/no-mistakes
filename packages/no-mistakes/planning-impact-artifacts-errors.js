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
