import { browserTest } from "../fixtures/relay";

browserTest.describe.configure({ mode: "serial", retries: 2 });
