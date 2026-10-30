import { createRequire } from "node:module";

const name = "require";

(module?.require)("node:fs");
(module?.["require"])("node:fs");
(module?.[name])("node:fs");
(module?.require!)("node:fs");
(createRequire?.(import.meta.url))("fs");

class Probe {
  #load = "node:fs";
  call() {
    return (this?.#load)("node:fs");
  }
}
