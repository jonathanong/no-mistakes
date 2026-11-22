type QueryCallback = () => unknown;
export function invoke00(callback: QueryCallback, payload: unknown) {
  return callback();
}
