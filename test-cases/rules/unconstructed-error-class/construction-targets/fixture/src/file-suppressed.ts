// no-mistakes-disable-file unconstructed-error-class -- This runtime registry is intentional.
export function fileSuppressed(Constructor: new () => unknown) {
  return new Constructor();
}
