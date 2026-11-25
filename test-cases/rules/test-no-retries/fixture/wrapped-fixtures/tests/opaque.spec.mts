import { test as opaqueTest } from "../helpers/opaque-wrapper.mts";

opaqueTest.describe.configure({ mode: "serial", retries: 2 });
