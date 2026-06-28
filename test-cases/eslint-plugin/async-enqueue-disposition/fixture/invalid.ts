import enqueueDefault, { enqueueEmail, enqueueSms as sendSms } from "@app/jobs";
import * as jobs from "@app/jobs";

const queue = require("@app/jobs");
const { enqueuePush } = require("@app/jobs");

export async function worker(id: string) {
  enqueueEmail(id);
  sendSms(id);
  jobs.enqueueSms(id);
  jobs["enqueueSms"](id);
  queue.enqueuePush(id);
  enqueuePush(id);
  enqueueDefault(id);
}

export async function fanout(items: string[]) {
  Promise.all(items.map((item) => enqueueEmail(item)));
  await Promise.all([wrap(enqueueEmail(items[0]))]);
  await Promise.all([[enqueueEmail(items[0])]]);
  await Promise.all([{ job: enqueueEmail(items[0]) }]);
  await Promise.all(items.forEach((item) => enqueueEmail(item)));
  await Promise.all(items.map((item) => [enqueueEmail(item)]));
  await Promise.all(
    items.map((item) => {
      enqueueEmail(item);
    }),
  );
  const callbacks = items.map((item) => {
    return enqueueEmail(item);
  });
  await Promise.all(callbacks);
}

export function laterRequire(id: string) {
  enqueueLater(id);
}

const { enqueueLater } = require("@app/jobs");

export function inlineRequire(id: string) {
  require("@app/jobs").enqueueEmail(id);
}
