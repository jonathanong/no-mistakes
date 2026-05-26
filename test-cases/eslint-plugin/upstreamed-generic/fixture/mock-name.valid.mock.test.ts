import { vi } from "vitest";

const mockClient = {
  send: vi.fn<VitestLooseMock>().mockResolvedValue({}),
};

vi.stubGlobal("crypto", undefined);
vi["stubGlobal"]("crypto", undefined);
