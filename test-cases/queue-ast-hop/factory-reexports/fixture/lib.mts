export function createQueue(name: string) {
  return { name };
}

export function createEnqueueFunction(_queue: object, _job: string) {
  return (_data: object) => undefined;
}

export function createWorker(
  _name: string,
  _processor: (job: { name: string }) => void,
) {
  return {};
}
