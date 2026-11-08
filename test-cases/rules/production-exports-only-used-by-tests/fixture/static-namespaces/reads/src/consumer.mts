import * as ns from "./worker.mjs";
const first = ns.work; const second = ns["work"]; first(); second();
