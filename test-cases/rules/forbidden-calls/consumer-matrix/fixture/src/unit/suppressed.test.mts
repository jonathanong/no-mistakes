export function suppressedTimer() {
  // no-mistakes-disable-next-line forbidden-calls: fixture proves findings-only suppression
  setTimeout(() => {}, 1);
}
