declare function getErrorCtor(): new () => Error;
declare const extra: Record<string, unknown>;
export const RateLimitError = getErrorCtor();
export let MutableRateLimitError = Error;
export { RateLimitError as ExternalReexport } from "third-party-queue-wrapper";
export class DynamicBaseError extends getErrorCtor() {}
export const SpreadHelper = { ...extra };
export const ComputedHelper = { [Symbol.toStringTag]: 1 };
export const OpaquePropHelper = { RateLimitError };
export * from "third-party-queue-wrapper";
