declare const handlers: Record<string, Function>;
export function process(job: { name: string }) {
  if (job.name === 'known') return;
  // The literal branch does not exhaust this dynamic dispatcher.
  return handlers[job.name]();
}
