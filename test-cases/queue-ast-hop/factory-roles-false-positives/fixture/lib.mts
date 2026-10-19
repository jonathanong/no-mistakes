export function createQueue(name: string) {
  return { name };
}

export function createWorker(
  _name: string,
  _processor: (job: { name: string; id: string }) => void,
) {
  return {};
}
