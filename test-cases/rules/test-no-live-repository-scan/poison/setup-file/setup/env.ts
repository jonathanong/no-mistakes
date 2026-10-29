// Outside the rule include, but it runs before every test body.
import { beforeEach } from "vitest";

beforeEach(() => {
  process.env.POISON_DIR = "./src";
});
