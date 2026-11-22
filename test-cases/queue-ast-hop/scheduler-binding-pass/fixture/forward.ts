import { Queue, Worker } from "bullmq";

const widgets = new Queue("widgets");
new Worker("widgets", processWidget);

function configure(options: object) {
  widgets.upsertJobScheduler("unknown", {}, { opts: options });
}

widgets.upsertJobScheduler("first", {}, firstTemplate);
widgets["upsertJobSched\u0075ler"]("second", {}, secondTemplate);

const firstTemplate = { opts: { ...firstOptions } };
const firstOptions = { delay: 1 };
const secondTemplate = { opts: { parent: { id: "widget" } } };

function processWidget(task: { name: string }) {
  return task.name === "pack";
}

void configure;
