import { localTest } from "./conditional-base";

export const chainedTest = process.env.CI ? localTest : localTest.extend({});
