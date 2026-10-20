declare const handlers: Record<string, Function>;
declare const field: string;
export function process(job: { name: string }) {
  if (job.name === 'known') return;
  // An unresolved property can be the job name, so dispatch remains open.
  return handlers[job[field]]();
}
