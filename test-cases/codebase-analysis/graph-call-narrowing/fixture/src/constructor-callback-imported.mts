import { Service } from "./constructor-callback-ctor.mts";

async function target() {
  await import("./constructor-callback-loaded.mts");
}

new Service(target);
