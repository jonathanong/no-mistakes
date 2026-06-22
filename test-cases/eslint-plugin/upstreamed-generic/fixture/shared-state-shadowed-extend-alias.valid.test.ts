import { test } from "vitest";

const shared: string[] = [];
const spec = test.extend({});

function run(spec: (name: string, callback: () => void) => void) {
  spec("case", () => {
    shared.push("value");
  });
}

run((_name, callback) => callback());
