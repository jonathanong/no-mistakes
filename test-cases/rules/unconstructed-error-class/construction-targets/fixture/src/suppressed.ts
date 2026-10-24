export function suppressed(Constructor: new () => unknown) {
  // no-mistakes-disable-next-line unconstructed-error-class -- Runtime constructor is intentional.
  return new Constructor();
}
