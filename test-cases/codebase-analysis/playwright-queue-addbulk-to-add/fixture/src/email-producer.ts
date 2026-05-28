import { emailQueue } from "./queues";

export async function enqueueGreet(userId: string) {
  // The post-diff state refactors `addBulk([{ name: 'greet' }])` into a
  // single `.add('greet', ...)` call. The diff scanner must NOT register
  // `greet` as a removed job because the `+` side still references it
  // through the `.add` call.
  await emailQueue.add("greet", { userId });
}
