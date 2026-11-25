import { createRunner } from "./runner";
function unrelated() {
  return Object.assign(createRunner(), {});
}

const suite = unrelated();

suite.describe.serial("ordered", () => {});
suite.describe.configure({ retries: 2 });
