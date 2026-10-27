import type { fnImportType, KlassType, IFace, Status, TypeOnlyLevel } from "./values.mjs";
import { type constInline, type letValue, type varValue } from "./values.mjs";
import type * as Ns from "./values.mjs";
import type { fnChain } from "./chain.mjs";
export type Refs = [
  typeof fnImportType, typeof constInline, typeof letValue, typeof varValue,
  KlassType, IFace, Status, TypeOnlyLevel, typeof Ns.fnNamespace, typeof fnChain,
  typeof import("./values.mjs").fnQualified,
  import("./values.mjs").fnImportQualified,
];
