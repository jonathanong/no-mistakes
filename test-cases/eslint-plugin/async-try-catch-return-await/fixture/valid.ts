import { handleRateLimit, handleRateLimit as onRateLimit } from "@app/rate-limit";
import * as rateLimit from "@app/rate-limit";

const handlers = require("@app/rate-limit");
const { handleRateLimit: handleRequiredRateLimit } = require("@app/rate-limit");

async function request() {
  return "ok";
}

export async function direct() {
  try {
    await Promise.all([, request()]);
    return await request();
  } catch (error) {
    handleRateLimit(error);
  }
}

export async function finallyOnly() {
  try {
    await request();
  } finally {
    console.info("done");
  }
}

export async function alias() {
  try {
    const { value } = request();
    void value;
    const result = await request();
    return result;
  } catch (error) {
    onRateLimit(error);
  }
}

export async function namespaceHandler() {
  try {
    return await request();
  } catch (error) {
    rateLimit.handleRateLimit(error);
  }
}

export async function requireHandler() {
  try {
    return await request();
  } catch (error) {
    handlers.handleRateLimit(error);
  }
}

export async function destructuredRequireHandler() {
  try {
    return await request();
  } catch (error) {
    handleRequiredRateLimit(error);
  }
}

export async function nestedIsIgnored() {
  try {
    function nested() {
      return request();
    }
    return await request();
  } catch (error) {
    function nestedCatch() {
      handleRateLimit(error);
    }
    console.error(error);
  }
}

export async function unmatchedHandler() {
  try {
    return request();
  } catch (error) {
    console.error(error);
  }
}

export function nonAsync() {
  try {
    return request();
  } catch (error) {
    handleRateLimit(error);
  }
}
