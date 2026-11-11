import fs from "node:fs";
import { vi } from "vitest";
const original = fs.readdir;
vi.spyOn(fs, "readdir").mockImplementation((...args) => original(...args));
