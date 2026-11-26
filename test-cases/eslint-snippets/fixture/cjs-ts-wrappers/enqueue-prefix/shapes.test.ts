import { vi } from "vitest";
import vt from "vitest";
import { enqueueJob as job } from "@example/jobs";

const { "enqueueJob": quoted, enqueueOther, ...rest } = require("@example/jobs");
vi.mocked(quoted);
vi.mocked(enqueueOther);
void rest;

const { ["enqueueTask"]: computed } = require("@example/jobs");
vi.mocked(computed);

const { enqueueJob: nested } = require("@example/jobs");
void nested;
const { enqueueJob: { run } } = require("@example/jobs");
void run;

const ns = require("@example/jobs");
const { enqueueJob: fromNs } = ns;
vi.mocked(fromNs);

(0, ns).vi.mocked(quoted);

vt.vi.mocked(job);

const { ...onlyRest } = require("vitest");
onlyRest.mocked(quoted);

var api;
var api = require("vitest");
api.vi.mocked(job);
