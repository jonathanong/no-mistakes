declare const client: {
  responses: { create: (options?: object) => void };
  chat: { responses: { create: (options?: object) => void } };
};
declare const agent: { responses: { create: (options?: object) => void } };

createResponse();
createOther();
other();
client.responses.create();
agent.responses.create();
client.chat.responses.create();
responses.create();
