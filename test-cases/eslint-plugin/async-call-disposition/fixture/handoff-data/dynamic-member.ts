import { enqueue } from '@data/jobs';
declare const key: string;
async function consumer(options: { enqueue: typeof enqueue }) {
  // Unresolved keys are rejected before closed-shape missing-property facts.
  const hidden = options[key];
  hidden();
  await options.enqueue();
}
const wrapped: typeof enqueue = () => enqueue();
void consumer({ enqueue: wrapped });
