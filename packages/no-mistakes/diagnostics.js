"use strict";

const { decorateInvocationError } = require("./invocation-errors");
const MARKER = "\nno-mistakes-diagnostics-json: ";
const EXCLUDED = [
  "collectDiagnostics",
  "version",
  "diagnosticsSnapshot",
  "writePlanningImpactArtifacts",
  "createWorkflowTopologyIndex",
  "testsComment",
  "testsGraphMermaid",
];
let contexts;

function diagnosticsError(error) {
  const message = error && typeof error.message === "string" ? error.message : "";
  const at = message.lastIndexOf(MARKER);
  if (at >= 0) {
    error.message = message.slice(0, at);
    try {
      error.diagnostics = JSON.parse(message.slice(at + MARKER.length));
    } catch {
      // Preserve the authoritative native error when diagnostic metadata is malformed.
    }
  }
  return decorateInvocationError(error);
}

async function callJson(fn, options) {
  const context = contexts?.getStore();
  const value = options || {};
  const request = !context
    ? value
    : Array.isArray(value) || typeof value !== "object"
      ? { __collectDiagnostics: true, __diagnosticsInput: value }
      : { ...value, __collectDiagnostics: true };
  const input = Buffer.from(JSON.stringify(request));
  try {
    const response = JSON.parse(await fn(input));
    if (!context) return response;
    if (
      !response ||
      !Object.hasOwn(response, "diagnostics") ||
      !Object.hasOwn(response, "result")
    ) {
      throw new Error("native response did not include requested diagnostics");
    }
    context.diagnostics = response.diagnostics;
    return response.result;
  } catch (error) {
    throw context ? diagnosticsError(error) : decorateInvocationError(error);
  }
}

async function collectDiagnostics(api, operation, ...args) {
  if (
    typeof operation !== "string" ||
    EXCLUDED.includes(operation) ||
    !Object.hasOwn(api, operation) ||
    typeof api[operation] !== "function"
  ) {
    throw new TypeError(`unsupported diagnostics operation: ${String(operation)}`);
  }
  if (!contexts) {
    const { AsyncLocalStorage } = require("node:async_hooks");
    contexts = new AsyncLocalStorage();
  }
  const context = {};
  return contexts.run(context, async () => {
    const result = await api[operation](...args);
    return { result, diagnostics: context.diagnostics };
  });
}

module.exports = { callJson, collectDiagnostics };
