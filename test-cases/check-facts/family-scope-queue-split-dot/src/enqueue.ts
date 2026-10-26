import { emailsQueue } from "./queues";

type Welcome = { userId: string };

// Split the member access across lines/comments; the prefilter must still
// seed this importer. Adjacent member calls live in family-scope-queue-generic.
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
