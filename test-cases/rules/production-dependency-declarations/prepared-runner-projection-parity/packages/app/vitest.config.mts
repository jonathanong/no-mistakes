import { sharedSettings } from "./runner-helper.mts";
import { typedSettings } from "./typed-runner.js";

export default { test: { ...sharedSettings, ...typedSettings } };
