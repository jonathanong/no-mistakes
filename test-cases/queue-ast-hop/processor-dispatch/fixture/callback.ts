const items = [1];
export default function process(job: { name: string }) {
  // This callback does not read the job name, so it is not a second dispatcher.
  items.map((x) => x);
  if (job.name === 'imp-a') return;
}
