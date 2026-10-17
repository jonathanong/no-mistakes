// Many exports deliberately share a cyclic graph; traverse each global once.
const leaf = () => null;
const shared = [cycleA, leaf];
const cycleA = [cycleB];
const cycleB = [shared];
export function entry00() { return shared; }
export function entry01() { return shared; }
export function entry02() { return shared; }
export function entry03() { return shared; }
export function entry04() { return shared; }
export function entry05() { return shared; }
export function entry06() { return shared; }
export function entry07() { return shared; }
export function entry08() { return shared; }
export function entry09() { return shared; }
export function entry10() { return shared; }
export function entry11() { return shared; }
export function entry12() { return shared; }
export function entry13() { return shared; }
export function entry14() { return shared; }
export function entry15() { return shared; }
export function entry16() { return shared; }
export function entry17() { return shared; }
export function entry18() { return shared; }
export function entry19() { return shared; }
export function entry20() { return shared; }
export function entry21() { return shared; }
export function entry22() { return shared; }
export function entry23() { return shared; }
export function entry24() { return shared; }
export function entry25() { return shared; }
export function entry26() { return shared; }
export function entry27() { return shared; }
export function entry28() { return shared; }
export function entry29() { return shared; }
export function entry30() { return shared; }
export function entry31() { return shared; }
export function entry32() { return shared; }
export function entry33() { return shared; }
export function entry34() { return shared; }
export function entry35() { return shared; }
export function entry36() { return shared; }
export function entry37() { return shared; }
export function entry38() { return shared; }
export function entry39() { return shared; }
export function entry40() { return shared; }
export function entry41() { return shared; }
export function entry42() { return shared; }
export function entry43() { return shared; }
export function entry44() { return shared; }
export function entry45() { return shared; }
export function entry46() { return shared; }
export function entry47() { return shared; }
export function entry48() { return shared; }
export function entry49() { return shared; }
export function entry50() { return shared; }
export function entry51() { return shared; }
export function entry52() { return shared; }
export function entry53() { return shared; }
export function entry54() { return shared; }
export function entry55() { return shared; }
export function entry56() { return shared; }
export function entry57() { return shared; }
export function entry58() { return shared; }
export function entry59() { return shared; }
export function entry60() { return shared; }
export function entry61() { return shared; }
export function entry62() { return shared; }
export function entry63() { return shared; }
// Preserve the existing conservative direct-return behavior even for a parameter.
function directShadow(leaf: unknown) { return leaf; }
// Following an unshadowed global resets the calling function's lexical shadows.
function globalScopeReset(leaf: unknown) { if (true) return leafOwner; }
function leafOwner() { if (true) return leaf; }
// A nested direct return keeps the enclosing function's parameter scope.
function nestedScope(leaf: unknown) { if (true) return () => leaf; }
