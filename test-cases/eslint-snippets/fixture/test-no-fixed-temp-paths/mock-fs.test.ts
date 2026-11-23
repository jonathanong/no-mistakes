import { writeFile } from "node:fs/promises";
import { vi as mocker } from "vitest";

await writeFile("/tmp/mock-file", "sample");
mocker.mock("fs");
