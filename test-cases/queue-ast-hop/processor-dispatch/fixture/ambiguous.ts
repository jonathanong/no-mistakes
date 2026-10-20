// A module import does not identify which of these callable bodies is the processor.
export function process(job: { name: string }) {
  if (job.name === 'known') return;
}
export function unrelatedHelper(job: { name: string }) {
  if (job.name === 'metadata') return;
}
