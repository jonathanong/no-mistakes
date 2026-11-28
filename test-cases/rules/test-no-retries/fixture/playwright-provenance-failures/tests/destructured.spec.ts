import { test as fixture } from "../fixtures/destructured";

const test = Math.random() > 0.5 ? fixture : undefined;
test?.describe.configure({ mode: "serial", retries: 2 });
