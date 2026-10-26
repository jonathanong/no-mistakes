import { test } from "@playwright/test";

// An undeclared bare locator() is recalled as the Playwright method on purpose.
locator(".a");

void test;
