import { test } from "@playwright/test";
import * as helpers from "./helpers";

function setup() {}
test("plain", setup);

function replaced() {}
replaced = other;
test("replaced", replaced);

helpers.mystery();
test.info().setTimeout(1);
test.info().slow();
test.info()[dynamic]();
