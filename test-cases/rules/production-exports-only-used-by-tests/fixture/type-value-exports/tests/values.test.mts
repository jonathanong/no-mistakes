import {
  fnImportType, constInline, fnQualified, fnImportQualified, fnNamespace, fnChain,
  letValue, varValue, KlassType, Status, ConstStatus,
} from "../src/values.mjs";
import { starTestOnly } from "../src/star.mjs";
import undeterminedDefault from "../src/default.mjs";
import type { IFace, TypeOnlyLevel } from "../src/values.mjs";
export const used = [
  fnImportType(), constInline, fnQualified(), fnImportQualified(), fnNamespace(),
  fnChain(), letValue, varValue, new KlassType(), Status.On, ConstStatus.On, undeterminedDefault(),
  starTestOnly(),
];
export type T = IFace;
export type Level = TypeOnlyLevel;
