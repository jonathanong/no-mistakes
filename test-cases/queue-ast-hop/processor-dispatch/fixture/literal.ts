export default function process(job: { name: string }) {
  if ('alpha' === job.name) return;
  switch (job.name) { case `beta`: return; }
}
