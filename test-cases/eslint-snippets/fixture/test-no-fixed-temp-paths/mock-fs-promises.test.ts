import * as jestGlobals from "@jest/globals";
import { readFile } from "node:fs/promises";

jestGlobals.jest.mock("fs/promises");
await readFile("/tmp/mock-file");
