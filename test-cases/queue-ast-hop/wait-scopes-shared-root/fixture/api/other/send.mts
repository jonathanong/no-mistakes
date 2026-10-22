// This wait stays outside both configured scopes even though its root matches.
export async function send(queue) {
  await queue.addAndWait("email", {});
}
