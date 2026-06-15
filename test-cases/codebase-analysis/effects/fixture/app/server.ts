import { makeCache } from "../lib/cache";
import { publish } from "../lib/pubsub";
import { start } from "../lib/a";

export function main() {
  makeCache();
  publish();
  start();
}
