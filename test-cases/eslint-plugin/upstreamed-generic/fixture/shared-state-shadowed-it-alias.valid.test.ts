import { it as spec } from "vitest";

const shared: string[] = [];

function run(spec: (name: string, callback: () => void) => void) {
  spec("case", () => {
    shared.push("value");
  });
}

run((_name, callback) => callback());
