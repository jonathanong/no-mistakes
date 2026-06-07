"use strict";

const DEFAULT_MAX_ATTEMPTS = 4;
const DEFAULT_RETRY_BASE_MS = 500;
const RETRY_MAX_DELAY_MS = 4_000;
const RETRYABLE_NETWORK_CODES = new Set([
  "ECONNRESET",
  "ECONNREFUSED",
  "ETIMEDOUT",
  "EAI_AGAIN",
  "ENETUNREACH",
  "EPIPE",
]);

class HttpError extends Error {
  constructor(url, statusCode) {
    super(`Download failed for ${url}: HTTP ${statusCode}`);
    this.name = "HttpError";
    this.statusCode = statusCode;
    this.retryable = isRetryableStatus(statusCode);
  }
}

function isRetryableStatus(statusCode) {
  if (typeof statusCode !== "number") return false;
  if (statusCode >= 500 && statusCode < 600) return true;
  return statusCode === 408 || statusCode === 429;
}

function isRetryableError(error) {
  if (!error) return false;
  if (error.retryable === true) return true;
  if (error.code && RETRYABLE_NETWORK_CODES.has(error.code)) return true;
  return false;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function parsePositiveInt(value, fallback) {
  if (value === undefined || value === null || value === "") return fallback;
  const n = Number(value);
  if (!Number.isFinite(n) || !Number.isInteger(n) || n < 1) return fallback;
  return n;
}

function computeBackoffMs(attempt, baseDelayMs, random = Math.random) {
  const exponential = baseDelayMs * 2 ** (attempt - 1);
  const capped = Math.min(exponential, RETRY_MAX_DELAY_MS);
  return Math.floor(random() * capped);
}

async function withRetry(fn, options = {}) {
  const maxAttempts = parsePositiveInt(
    options.maxAttempts ?? process.env.NO_MISTAKES_DOWNLOAD_MAX_ATTEMPTS,
    DEFAULT_MAX_ATTEMPTS,
  );
  const baseDelayMs = parsePositiveInt(
    options.baseDelayMs ?? process.env.NO_MISTAKES_DOWNLOAD_RETRY_BASE_MS,
    DEFAULT_RETRY_BASE_MS,
  );
  const delay = options.delay ?? sleep;
  const random = options.random ?? Math.random;
  const logger = options.logger ?? console;
  const describe = options.describe ?? (() => "download");

  let attempt = 0;
  for (;;) {
    attempt += 1;
    try {
      return await fn();
    } catch (error) {
      if (attempt >= maxAttempts || !isRetryableError(error)) {
        throw error;
      }
      const waitMs = computeBackoffMs(attempt, baseDelayMs, random);
      logger.warn(
        `no-mistakes: retrying ${describe()} after ${error.message} (attempt ${attempt + 1}/${maxAttempts}, waiting ~${waitMs}ms)`,
      );
      await delay(waitMs);
    }
  }
}

module.exports = {
  HttpError,
  computeBackoffMs,
  isRetryableError,
  isRetryableStatus,
  parsePositiveInt,
  withRetry,
};
