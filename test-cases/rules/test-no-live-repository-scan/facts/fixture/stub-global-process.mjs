import { stubGlobal } from "vitest";

stubGlobal("process", { env: {} });
const runner = { stubGlobal };
runner.stubGlobal("process", { env: {} });
runner["stubGlobal"]("process", { env: {} });
runner?.stubGlobal?.("process", { env: {} });
const name = "process";
stubGlobal(name, { env: {} });
