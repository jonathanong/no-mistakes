import { test as fixture } from "../fixtures/unrelated";

const test = Math.random() > 0.5 ? fixture : undefined;
test?.describe.configure({ mode: "serial", retries: 2 });
