export function loadReachable() {
  // no-mistakes-disable-next-line test-no-unmocked-dynamic-imports: reachable import is intentional
  return import('./leaf.mts')
}
