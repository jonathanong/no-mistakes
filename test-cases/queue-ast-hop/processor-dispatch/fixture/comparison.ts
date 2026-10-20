declare const dynamicName: string;
export function process(job: { name: string }) {
  if (job.name === 'known') return;
  if (job.name === dynamicName) return;
}
