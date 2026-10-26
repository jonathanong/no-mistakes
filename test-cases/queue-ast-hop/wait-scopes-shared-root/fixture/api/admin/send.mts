export async function send(queue) {
  await queue.addAndWait("email", {});
}
