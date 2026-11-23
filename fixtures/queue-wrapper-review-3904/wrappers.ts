export function makeEnqueue(_definition: object) {
  return (_data: unknown, _options?: object) => {};
}
export function makeStringEnqueue(_name: string) {
  return (_data: unknown, _options?: object) => {};
}
export function makeNamedEnqueue(_queue: unknown, _name: string) {
  return (_data: unknown, _options?: object) => {};
}
export function enqueueLegacy(_queue: unknown, _name: string, _options?: object) {}
export function createQueue(_name: string) { return {}; }
export function createWorker(_name: string, _processor: () => void) {}
