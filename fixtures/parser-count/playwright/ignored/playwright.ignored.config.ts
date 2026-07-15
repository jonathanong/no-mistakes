import { configure as makeConfig } from "../playwright.helper";

export const configure = "ignored-config-collision";

export default makeConfig({
  testDir: "../tests",
  testMatch: "**/*.ts",
  use: { testIdAttribute: "data-testid" },
});
