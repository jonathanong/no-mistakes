"use strict";

const ERROR_JSON_MARKER = "\nno-mistakes-error-json: ";

// Lock and deadline failures carry a machine-readable report after a marker line.
// Expose it as `error.code` ("NO_MISTAKES_LOCK_BUSY", "NO_MISTAKES_LOCK_TIMEOUT",
// "NO_MISTAKES_DEADLINE") and `error.details` without changing the message text.
function decorateInvocationError(error) {
  const message = error && typeof error.message === "string" ? error.message : "";
  const at = message.indexOf(ERROR_JSON_MARKER);
  if (at < 0) return error;
  try {
    const details = JSON.parse(message.slice(at + ERROR_JSON_MARKER.length));
    error.message = message.slice(0, at);
    error.code = `NO_MISTAKES_${String(details.kind).toUpperCase()}`;
    error.details = details;
  } catch {
    // Keep the native error as-is when the report cannot be parsed.
  }
  return error;
}

module.exports = { decorateInvocationError };
