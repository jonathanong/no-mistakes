class BaseAppError extends Error {}
export class RateLimitError extends BaseAppError {}
export const CustomFnRateLimitError = () => new Error("custom");
export const CustomHelper = { RateLimitError };
export function CustomDeclaredError() {
  return new Error("custom");
}
export default class DefaultAppError {}
