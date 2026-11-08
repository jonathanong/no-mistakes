import * as ns from "./worker.mjs";
function read(ns) { ns.work(); } read({ work() {} });
