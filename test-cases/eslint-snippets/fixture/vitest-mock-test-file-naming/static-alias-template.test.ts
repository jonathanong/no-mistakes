import { vi } from "vitest";
import * as runner from "vitest";
const ns = runner; const { [`vi`]: api } = ns; api.mock?.("./a");
