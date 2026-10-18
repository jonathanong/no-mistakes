import { createResponse } from "./local-api";

function createResponseShadow() {
  return;
}

export function shadowed(
  createResponse: (options?: object) => void,
  window: { createResponse?: (options?: object) => void },
  globalThis: { createResponse?: (options?: object) => void },
) {
  createResponse();
  window.createResponse?.();
  globalThis.createResponse?.();
  function inner() {
    const createResponse = () => {};
    createResponse();
  }
}

function classShadow() {
  class createResponse {}
  createResponse();
}

createResponse();
createResponseShadow();
