declare const handlers: Record<string, Function>;
export function process(job: { name: string }) {
  if (job.name === 'known') return;
  return handlers[job['name']]();
}
