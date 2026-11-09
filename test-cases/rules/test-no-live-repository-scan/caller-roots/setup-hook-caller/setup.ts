import { list } from "./src/list";
import { beforeEach } from "vitest";

beforeEach(() => list(process.cwd()));
