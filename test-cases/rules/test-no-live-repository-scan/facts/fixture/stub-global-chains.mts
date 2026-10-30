import { stubGlobal } from "vitest";

stubGlobal();
const runner = { stubGlobal };
(runner?.stubGlobal)("process", { env: {} });
(runner?.["stubGlobal"])("process", { env: {} });
(runner?.stubGlobal!)("process", { env: {} });
(stubGlobal?.("process"))("process", { env: {} });

class Probe {
  #load = stubGlobal;
  call() {
    return (this?.#load)("process", { env: {} });
  }
}
