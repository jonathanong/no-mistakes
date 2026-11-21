import { FlowProducer } from "bullmq";

declare const mystery: object;
declare const dynamicName: string;
declare const mysteryKids: object[];
declare function makeItem(): object;

const flow = new FlowProducer();

flow.add({
  name: "root",
  queueName: "widgets",
  data: {},
  opts: { backoff: 1 },
  children: [
    { name: "child", queueName: "widgets", data: {} },
    { name: "orphan", queueName: "missing", data: {}, opts: {} },
    ...mysteryKids,
    makeItem(),
  ],
});

flow.add({
  name: "hidden",
  queueName: "widgets",
  data: {},
  ...mystery,
});

flow.add({
  name: "dynamic",
  queueName: dynamicName,
  data: {},
  opts: {},
});

flow.add({
  name: "open-children",
  queueName: "widgets",
  data: {},
  opts: { backoff: 1 },
  children: mystery,
});

flow.add(mystery);
