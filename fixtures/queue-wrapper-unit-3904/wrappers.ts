export function makeEnqueue(_definition: object, _defaults?: object) {
  return (_data?: unknown, _options?: object) => {};
}

export function enqueuePositional(_queue: object, _name: string, _options?: object) {}

export function createQueue(_name: string) {
  return {};
}

export function createWorker(_name: string, _processor: (job: { name: string }) => void) {}

export function enqueueProperty(_queue: object, _name: string, _envelope: unknown, _defaults?: object) {
  return (_data: unknown, _options?: object) => {};
}
