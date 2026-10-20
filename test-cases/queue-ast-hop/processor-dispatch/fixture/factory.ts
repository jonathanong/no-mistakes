export function makeWorker(queue: string, processor: unknown) {
  return { queue, processor };
}
