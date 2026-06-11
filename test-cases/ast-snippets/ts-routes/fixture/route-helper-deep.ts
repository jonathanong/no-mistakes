function passthrough(value: string): string {
  return value;
}
export function deepHref(value: string): string {
  return `/deep/${passthrough(passthrough(passthrough(passthrough(passthrough(value)))))}`;
}
