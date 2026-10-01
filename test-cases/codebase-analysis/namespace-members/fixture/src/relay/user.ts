import Direct from "./hub";
import Gate from "./stars";
import { Hub as ViaStar } from "./stars";
import { Hub, Renamed, Spare, Hinge, plain, Package } from "./barrel";

// Each of these reaches a namespace of `hub.ts` through a re-export.
export const named = () => new Hub.HubClass();
// The same export again is answered from the lookup cache.
export const repeated = () => new Hub.HubClass();
export const renamed = () => new Renamed.OtherClass();
export const spare = () => new Spare.SpareClass();
export const hinge = () => new Hinge.PivotClass();
export const direct = () => new Direct.PivotClass();
export const viaStar = () => new ViaStar.HubClass();

// `export *` never carries a default export, and the last two are not
// namespaces: none of them reaches a class.
export const gate = () => new Gate.PivotClass();
export const notNamespace = () => new plain.Thing();
export const external = () => new Package.Thing();
