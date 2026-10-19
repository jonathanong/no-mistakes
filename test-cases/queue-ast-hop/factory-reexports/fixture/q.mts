import { createQueue } from "./lib.mts";
import { QN } from "./names.mts";

export const bg = createQueue(QN);
export const other = createQueue("other");
// Same-file type specifiers are not queue exports. A later value alias is.
export type { bg as BgOnly };
export { type bg as BgType, other as alsoOther };
