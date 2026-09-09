import { setTimeout as wait } from "node:timers";
import * as timers from "node:timers";
import { setTimeout as waitP } from "node:timers/promises";
import * as timersP from "node:timers/promises";
import { sleep } from "../helper.mts";
import { setTimeout as localTimeout } from "../clock.mts";

function delay() {
  setTimeout(() => {}, 1);
}

export function run() {
  setTimeout(() => {}, 1);
  wait(() => {}, 1);
  timers.setTimeout(() => {}, 1);
  void waitP(1);
  timersP.setTimeout(1);
  delay();
  sleep();
  localTimeout(() => {}, 1);
}
