import { vi } from "vitest";

vi.stubGlobal("crypto", undefined);
vi["stubGlobal"]("crypto", undefined);
