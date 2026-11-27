import { test } from "@playwright/test";
import * as kit from "my-kit";
import { helper } from "my-kit";

declare const other: unknown;
test.setTimeout.bind(other)(999_999);
kit.custom(1);
helper();
void test;
