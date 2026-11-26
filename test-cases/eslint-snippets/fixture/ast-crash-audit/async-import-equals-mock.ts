import { enqueue } from "@fixture/jobs";
import framework = require("vitest");

framework.vi.fn(() => enqueue());

import unrelated = require("@fixture/unrelated");
unrelated.vi.fn(() => enqueue());
