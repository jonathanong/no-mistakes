import { Queue, Worker } from "bullmq";

const LEVEL = 5;
const fnParams = new Queue("fnParams");
const arrowParams = new Queue("arrowParams");
const pattern = new Queue("pattern");
const switchScoped = new Queue("switchScoped");
const switchOuter = new Queue("switchOuter");
const crossCase = new Queue("crossCase");

new Worker("fnParams", async () => {});
new Worker("arrowParams", async () => {});
new Worker("pattern", async () => {});
new Worker("switchScoped", async () => {});
new Worker("switchOuter", async () => {});
new Worker("crossCase", async () => {});

declare const route: number;
declare function readPriority(): number;

export function send(result = fnParams.add("fnRush", {}, { priority: LEVEL })) {
  var LEVEL = readPriority();
  return [result, LEVEL];
}
fnParams.add("fnPlain", {});

export const arrow = (
  result = arrowParams.add("arrowRush", {}, { priority: LEVEL }),
) => {
  var LEVEL = readPriority();
  return [result, LEVEL];
};
arrowParams.add("arrowPlain", {});

export function sendPattern({ value = ((LEVEL: number) => LEVEL)(0) } = {}) {
  pattern.add("patternRush", {}, { priority: LEVEL });
  return value;
}
pattern.add("patternPlain", {});

switch ((switchOuter.add("inDiscriminant", {}, { priority: LEVEL }), route)) {
  case 0:
    switchScoped.add("inEarlierCase", {}, { priority: LEVEL });
    break;
  case 1:
    let LEVEL = readPriority();
    switchScoped.add("afterDeclaration", {}, { priority: LEVEL });
}
switchScoped.add("scopePlain", {});
switchOuter.add("afterSwitch", {}, { priority: LEVEL });
switchOuter.add("outerPlain", {});

switch (route) {
  case 0:
    const CROSS = 7;
    crossCase.add("localRush", {}, { priority: CROSS });
    break;
  case 1:
    crossCase.add("crossArm", {}, { priority: CROSS });
}
crossCase.add("crossPlain", {});
