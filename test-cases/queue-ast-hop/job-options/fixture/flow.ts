import { FlowProducer } from "bullmq";

declare const mystery: object;
declare const dynamicName: string;
declare const mysteryKids: object[];
declare function makeItem(): object;
declare function extra(): object;

const knownOpts = { opts: { backoff: 1 } };
const dataOnly = { data: { n: 1 } };

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

flow.add({
  name: "folded-inline",
  queueName: "widgets",
  data: {},
  ...{ opts: { backoff: 1 } },
});

flow.add({
  name: "folded-const",
  queueName: "widgets",
  data: {},
  ...knownOpts,
});

flow.add({
  name: "kept-opts",
  queueName: "widgets",
  data: {},
  opts: { backoff: 1 },
  ...dataOnly,
});

flow.add({
  name: "folded-parent",
  queueName: "widgets",
  opts: { backoff: 1 },
  data: {},
  ...{
    children: [{ name: "folded-child", queueName: "widgets", data: {} }],
  },
});

flow.add({
  name: "called-spread",
  queueName: "widgets",
  data: {},
  ...extra(),
  children: [{ name: "hidden-child", queueName: "widgets", data: {} }],
});
