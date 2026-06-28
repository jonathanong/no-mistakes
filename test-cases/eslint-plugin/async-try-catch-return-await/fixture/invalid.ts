import { handleRateLimit, handleRateLimit as onRateLimit } from "@app/rate-limit";
import * as rateLimit from "@app/rate-limit";

const handlers = require("@app/rate-limit");
const { handleRateLimit: handleRequiredRateLimit } = require("@app/rate-limit");

async function request() {
  return "ok";
}

class RequestJob {}

export async function direct() {
  try {
    return request();
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function alias() {
  try {
    const result = request();
    return result;
  } catch (error) {
    onRateLimit(error);
  }
}

export async function namespaceHandler() {
  try {
    return request();
  } catch (error) {
    rateLimit.handleRateLimit(error);
  }
}

export async function requireHandler() {
  try {
    return request();
  } catch (error) {
    handlers.handleRateLimit(error);
  }
}

export async function destructuredRequireHandler() {
  try {
    return request();
  } catch (error) {
    handleRequiredRateLimit(error);
  }
}

export async function typeWrappers() {
  try {
    const result = request() as Promise<string>;
    return result!;
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function satisfiesWrapper() {
  try {
    return request() satisfies Promise<string>;
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function branchReturn(useFallback: boolean) {
  try {
    return useFallback ? cachedRequest() : request();
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function constructorReturn() {
  try {
    return new RequestJob();
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function dynamicImportReturn() {
  try {
    return import("./worker");
  } catch (error) {
    handleRateLimit(error);
  }
}
