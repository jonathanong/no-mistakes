declare const dynamicName: string;
export function process(job: { name: string }) {
  switch (job.name) { case 'known': return; case dynamicName: return; }
}
