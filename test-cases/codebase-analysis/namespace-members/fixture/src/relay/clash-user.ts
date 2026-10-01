import { Clash } from "./clash";
import { Abroad } from "./clash-package";

// Each name is ambiguous in the barrel it comes from, so neither construction
// is known to build the namespace's class.
export const viaClash = () => new Clash.ClashClass();
export const viaAbroad = () => new Abroad.AbroadClass();
