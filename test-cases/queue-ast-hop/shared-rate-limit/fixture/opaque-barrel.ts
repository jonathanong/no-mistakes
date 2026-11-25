declare function getErrorCtor(): new () => Error;
export const RateLimitError = getErrorCtor();
export let MutableRateLimitError = Error;
export { RateLimitError as ExternalReexport } from "third-party-queue-wrapper";
