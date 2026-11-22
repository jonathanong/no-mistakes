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
  children: [{ name: "mystery-child", queueName: "widgets", data: {} }],
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
  children: [{ name: "const-child", queueName: "widgets", data: {} }],
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

const laterData = { data: { n: 2 } };
const template = { opts: { backoff: 1 } };
const alias = template;
const dataAlias = dataOnly;
const openBag = { ...mystery };
const emptyBag = {};

flow.add({
  name: "both-spreads",
  queueName: "widgets",
  data: {},
  ...knownOpts,
  ...laterData,
});

flow.add({
  name: "aliased",
  queueName: "widgets",
  data: {},
  ...alias,
});

flow.add({
  name: "data-alias",
  queueName: "widgets",
  opts: { backoff: 1 },
  data: {},
  ...dataAlias,
});

flow.add({
  name: "open-bag",
  queueName: "widgets",
  opts: { backoff: 1 },
  data: {},
  ...openBag,
});

flow.add({
  name: "empty-bag",
  queueName: "widgets",
  opts: { backoff: 1 },
  data: {},
  ...emptyBag,
});

flow.add({
  name: "computed-node",
  queueName: "widgets",
  data: {},
  [dynamicName]: { backoff: 1 },
  children: [{ name: "computed-child", queueName: "widgets", data: {} }],
});
