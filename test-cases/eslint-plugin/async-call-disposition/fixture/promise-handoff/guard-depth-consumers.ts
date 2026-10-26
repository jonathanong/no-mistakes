import type { Dependencies } from "./guard-types";
// Bounded proofs stop even when every forwarding hop has the same target type.
export function hop0(d: Dependencies) { hop1(d); }
function hop1(d: Dependencies) { hop2(d); }
function hop2(d: Dependencies) { hop3(d); }
function hop3(d: Dependencies) { hop4(d); }
function hop4(d: Dependencies) { hop5(d); }
function hop5(d: Dependencies) { hop6(d); }
function hop6(d: Dependencies) { hop7(d); }
function hop7(d: Dependencies) { hop8(d); }
function hop8(d: Dependencies) { hop9(d); }
function hop9(d: Dependencies) { hop10(d); }
function hop10(d: Dependencies) { hop11(d); }
function hop11(d: Dependencies) { hop12(d); }
function hop12(d: Dependencies) { hop13(d); }
function hop13(d: Dependencies) { hop14(d); }
function hop14(d: Dependencies) { hop15(d); }
function hop15(d: Dependencies) { hop16(d); }
function hop16(d: Dependencies) { hop17(d); }
function hop17(d: Dependencies) { hop18(d); }
function hop18(d: Dependencies) { hop19(d); }
function hop19(d: Dependencies) { hop20(d); }
function hop20(d: Dependencies) { hop21(d); }
function hop21(d: Dependencies) { hop22(d); }
function hop22(d: Dependencies) { hop23(d); }
function hop23(d: Dependencies) { hop24(d); }
function hop24(d: Dependencies) { hop25(d); }
function hop25(d: Dependencies) { hop26(d); }
function hop26(d: Dependencies) { hop27(d); }
function hop27(d: Dependencies) { hop28(d); }
function hop28(d: Dependencies) { hop29(d); }
function hop29(d: Dependencies) { hop30(d); }
function hop30(d: Dependencies) { hop31(d); }
function hop31(d: Dependencies) { hop32(d); }
function hop32(d: Dependencies) { hop33(d); }
function hop33(d: Dependencies) { void d.x([], undefined); }
export function aliasDepth(d: Dependencies) {
  // Stable aliases still consume the per-proof traversal budget.
  const a0 = d;
  const a1 = a0;
  const a2 = a1;
  const a3 = a2;
  const a4 = a3;
  const a5 = a4;
  const a6 = a5;
  const a7 = a6;
  const a8 = a7;
  const a9 = a8;
  const a10 = a9;
  const a11 = a10;
  const a12 = a11;
  const a13 = a12;
  const a14 = a13;
  const a15 = a14;
  const a16 = a15;
  const a17 = a16;
  const a18 = a17;
  const a19 = a18;
  const a20 = a19;
  const a21 = a20;
  const a22 = a21;
  const a23 = a22;
  const a24 = a23;
  const a25 = a24;
  const a26 = a25;
  const a27 = a26;
  const a28 = a27;
  const a29 = a28;
  const a30 = a29;
  const a31 = a30;
  const a32 = a31;
  const a33 = a32;
  void a33.x([], undefined);
}
