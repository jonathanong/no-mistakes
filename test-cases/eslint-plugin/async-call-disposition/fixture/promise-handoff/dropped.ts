import { enqueueEmail } from "@app/jobs";

export function run(dependencies: { x: typeof enqueueEmail }) {
  dependencies.x([], undefined);
}
