import { enqueueEmail } from "@app/jobs";

export async function run(dependencies: { x: typeof enqueueEmail }) {
  await dependencies.x([], undefined);
}
