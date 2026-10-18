import { createResponse } from "agent/node";

declare const client: { responses: { create: (options?: object) => void } };

export function run() {
  createResponse();
  client.responses.create();
}
