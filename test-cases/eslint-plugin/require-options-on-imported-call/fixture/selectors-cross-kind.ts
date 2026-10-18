import * as client from "agent";

client.create();
client.create({ signal: true });
client.create({ timeout: true });
globalThis.createResponse();
globalThis.createResponse({ signal: true });
