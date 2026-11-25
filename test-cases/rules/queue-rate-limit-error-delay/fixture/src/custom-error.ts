export interface ProviderLimitOptions {
  retryAfterMs?: number;
}

export class ProviderRateLimitError extends Error {
  readonly retryAfterMs?: number;

  constructor(message: string, options?: ProviderLimitOptions) {
    super(message);
    this.retryAfterMs = options?.retryAfterMs;
  }
}
