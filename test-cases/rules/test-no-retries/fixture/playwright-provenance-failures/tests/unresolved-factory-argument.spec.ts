import { createTest } from "../fixtures/factory";
import { baseURL } from "../fixtures/settings";

const test = createTest({ baseURL });
test.describe.configure({ mode: "serial", retries: 2 });
