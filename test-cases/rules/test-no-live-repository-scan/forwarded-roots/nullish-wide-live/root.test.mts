import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "bounded-"));
const project = { k0: null, k1: null, k2: null, k3: null, k4: null, k5: null, k6: null, k7: null, k8: null, k9: null, k10: null, k11: null, k12: null, k13: null, k14: null, k15: null, k16: null, k17: null, k18: null, k19: null, k20: null, k21: null, k22: null, k23: null, k24: null, k25: null, k26: null, k27: null, k28: null, k29: null, k30: null, k31: null, k32: null, root: process.cwd() };
readdirSync(project.root ?? root);
