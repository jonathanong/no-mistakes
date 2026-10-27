import {
  fnImportType, constInline, fnQualified, fnImportQualified, fnNamespace, fnChain,
  letValue, varValue, KlassType, Status,
} from "../src/values.mjs";
import undeterminedDefault from "../src/default.mjs";
import type { IFace } from "../src/values.mjs";
export const used = [
  fnImportType(), constInline, fnQualified(), fnImportQualified(), fnNamespace(),
  fnChain(), letValue, varValue, new KlassType(), Status.On, undeterminedDefault(),
];
export type T = IFace;
