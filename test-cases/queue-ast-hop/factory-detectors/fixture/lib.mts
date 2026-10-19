export function createQueue(name: string) {
  return { name, add() {}, addAndWait() {}, addBulk() {} };
}

export function createWorker(
  _name: string,
  _processor: (job: { name: string; id: string }) => void,
) {
  return {};
}

export function enqueueJob(
  _queue: object,
  _job: string,
  _data: object,
  _options: object,
) {
  return {};
}
