import { vi } from "vitest";

const mockClient = {
  send: vi.fn<VitestLooseMock>().mockResolvedValue({}),
};

vi.mock("./dep");
vi.doMock("./other");
vi.unmock("./dep");
vi["doUnmock"]("./other");
