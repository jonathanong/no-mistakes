export function disabledLineImport() {
  // no-mistakes-disable-next-line test-no-unmocked-dynamic-imports
  return import('./dynamic-leaf.mts')
}
