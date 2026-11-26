declare function getErrorCtor(): new () => Error;
declare const extra: Record<string, unknown>;
declare const unknownObj: { RateLimitError(): Error; Worker: { RateLimitError(): Error } };
export const RateLimitError = getErrorCtor();
export let MutableRateLimitError = Error;
export { RateLimitError as ExternalReexport } from "third-party-queue-wrapper";
export class DynamicBaseError extends getErrorCtor() {}
export const SpreadHelper = { ...extra };
export const ComputedHelper = { [Symbol.toStringTag]: 1 };
export const OpaquePropHelper = { RateLimitError };
export const OpaqueMemberAlias = unknownObj.RateLimitError;
export const OpaqueWorkerMemberAlias = unknownObj.Worker.RateLimitError;
export const OpaqueCallMemberAlias = getErrorCtor().Worker.RateLimitError;
export * from "third-party-queue-wrapper";
