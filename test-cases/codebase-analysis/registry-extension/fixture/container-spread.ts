import { Alpha } from "./alpha";
import { Beta } from "./beta";
import { base } from "./base";

// Spread entries are skipped; Alpha/Beta are the detected entries.
export default { ...base, alpha: Alpha, beta: Beta };
