import { test } from "vitest";
import * as v from "vitest";

const t = test;
const suite = v.describe;

t.sequential("alias", () => {});
suite.sequential("suite", () => {});
