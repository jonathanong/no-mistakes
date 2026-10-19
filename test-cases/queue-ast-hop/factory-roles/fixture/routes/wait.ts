import { background } from "../queues";

export async function route() {
  await background.addAndWait("page", {});
}
