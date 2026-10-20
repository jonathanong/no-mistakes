// The actual default processor has no dispatch. A helper's names must not close it.
export default function process(task: { data: unknown }) {
  return task.data;
}
export function helper(job: { name: string }) {
  if (job.name === 'phantom') return;
}
