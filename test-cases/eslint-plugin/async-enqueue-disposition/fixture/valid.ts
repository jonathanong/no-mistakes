import enqueueDefault, { enqueueEmail, enqueueSms as sendSms } from "@app/jobs";
import * as jobs from "@app/jobs";
import * as otherJobs from "@other/jobs";

const queue = require("@app/jobs");
const { enqueuePush } = require("@app/jobs");
const { ...rest } = require("@app/jobs");

export async function worker(items: string[]) {
  await enqueueEmail(items[0]);
  return sendSms(items[1]);
}

export async function fanout(items: string[]) {
  await Promise.all(items.map((item) => enqueueEmail(item)));
  return Promise.all(items.map((item) => jobs.enqueueSms(item)));
}

export async function explicitDiscard(id: string) {
  void enqueuePush(id);
  void Promise.all([queue.enqueuePush(id), enqueueDefault(id)]);
}

export async function blockCallback(items: string[]) {
  await Promise.all(
    items.map((item) => {
      return enqueueEmail(item);
    }),
  );
}

export async function shadowed(enqueueEmail: (id: string) => void, id: string) {
  enqueueEmail(id);
}

export async function unmatched(id: string) {
  saveEmail(id);
  otherJobs.enqueueEmail(id);
  jobs.notConfigured(id);
  jobs[dynamicMethod](id);
  getJobs().enqueueEmail(id);
  rest.enqueueEmail(id);
}
