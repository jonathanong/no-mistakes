import { starred } from "./starred";
import { inner } from "./wrapped";
import { Ghost } from "./ghost";
import { Loop } from "./loop-a";
import { Shared } from "./dup";
import { Elsewhere } from "./nowhere";

// Each construction names a namespace the chain of exports does not lead to, so
// it could build any class of the modules the chain passes through.
export const viaStarred = () => new starred.Starred.StarredClass();
export const viaWrapped = () => new inner.Wrapped.WrappedClass();
export const viaGhost = () => new Ghost.Thing();
export const viaLoop = () => new Loop.Thing();
export const viaShared = () => new Shared.Thing();
export const viaElsewhere = () => new Elsewhere.Thing();
