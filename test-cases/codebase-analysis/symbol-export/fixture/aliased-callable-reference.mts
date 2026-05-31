import { alpha } from "./source.mts";

function helper() {
  return alpha();
}

function run() {
  return helper;
}

export { run as api };
