import type { TestNoConfigMirrorAssertionOptions, TestNoConfigMirrorParser } from "../src/index.js";
import pluginRequire = require("../src/index.js");

export const configMirrorParser: TestNoConfigMirrorParser = {
  module: "smol-toml",
  name: "parse",
};
export const configMirrorOptions: TestNoConfigMirrorAssertionOptions = {
  fixtureDirs: ["fixtures", "testdata"],
  parsers: [configMirrorParser],
};
export const defaultConfigMirrorOptions: pluginRequire.TestNoConfigMirrorAssertionOptions = {};
export const invalidConfigMirrorFixtureDirs: TestNoConfigMirrorAssertionOptions = {
  // @ts-expect-error Fixture directories must be strings.
  fixtureDirs: [1],
};
export const invalidConfigMirrorParser: TestNoConfigMirrorAssertionOptions = {
  // @ts-expect-error Configured parsers require both module and name.
  parsers: [{ module: "smol-toml" }],
};
