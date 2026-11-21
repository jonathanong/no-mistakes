export class WidgetError extends Error {}

export function failOrder(message: string): WidgetError {
  return new WidgetError(message);
}
