function createError(): new (message: string, opts?: { retryAfterMs?: number }) => Error {
  return class extends Error {};
}

export const ProviderRateLimitError = createError();
