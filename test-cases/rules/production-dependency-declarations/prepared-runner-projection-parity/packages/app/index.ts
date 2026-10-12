import { sharedSettings } from "./runner-helper.mts";
import { typedSettings } from "./typed-runner.js";
export const settings = { ...sharedSettings, ...typedSettings };
