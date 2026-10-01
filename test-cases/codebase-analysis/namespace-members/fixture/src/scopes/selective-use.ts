import * as barrel from "./selective-barrel";

// The whole barrel is a value: the namespaces it exports escape, and the ones
// its source keeps to itself do not.
export const everything = barrel;
