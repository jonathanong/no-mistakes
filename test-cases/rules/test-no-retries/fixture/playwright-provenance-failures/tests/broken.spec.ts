import { test } from "../fixtures/broken";

test.describe.configure({ mode: "serial", retries: 2 });
