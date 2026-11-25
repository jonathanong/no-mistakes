class BaseAppError extends Error {}
const { destructured } = { destructured: 1 };
void destructured;
export class RateLimitError extends BaseAppError {}
export const CustomFnRateLimitError = () => new Error("custom");
export const CustomHelper = { RateLimitError };
export const ExprClassError = class extends BaseAppError {};
export function CustomDeclaredError() {
  return new Error("custom");
}
export default class DefaultAppError {}
