import { createSuite } from "../fixtures/plain-factory";
import { baseURL } from "../fixtures/settings";

const suite = createSuite({ baseURL });
suite.describe.configure({ mode: "serial", retries: 2 });
