import { vi as v } from "vitest";

const mockClient = {
  send: v.fn<VitestLooseMock>().mockResolvedValue({}),
};

v["mock"]("./dep");
