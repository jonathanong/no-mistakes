import { test as base } from "@playwright/test";

declare const useBase: boolean;
const replacement = { describe: { configure(_options: object) {} } };
export const createTest = () => (useBase ? base : replacement);
