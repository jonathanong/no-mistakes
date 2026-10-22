import { emailsQueue } from "./queues";

type Welcome = { userId: string };

// Keep whitespace/comments between `.` and `add`: the prefilter must still
// seed this importer. Adjacent `.add` is covered by family-scope-queue-generic.
export function newlineAdd(data: Welcome) {
  return emailsQueue.
    add("sendWelcome", data);
}

export function spaceAdd(data: Welcome) {
  return emailsQueue. add("sendWelcome", data);
}

export function commentAdd(data: Welcome) {
  return emailsQueue./*c*/add("sendWelcome", data);
}
