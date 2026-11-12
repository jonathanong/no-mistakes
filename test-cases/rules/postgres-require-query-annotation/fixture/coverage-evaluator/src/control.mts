import { write } from "@app/db";
function truth() { if (true) return "SELECT 1"; return "SELECT 2"; }
function falsity() { if (false) return "SELECT 1"; return "SELECT 2"; }
function zero() { if (0) return "SELECT 1"; return "SELECT 2"; }
function nonzero() { if (2) return "SELECT 1"; return "SELECT 2"; }
function comparison() { if (2 < 3) return "SELECT 1"; return "SELECT 2"; }
function reversed() { if (2 >= 3) return "SELECT 1"; return "SELECT 2"; }
function early() { if (true) return "SELECT 1"; opaqueMutation(); return "SELECT 2"; }
function same(flag) { if (flag) return "SELECT 3"; return "SELECT 3"; }
function different(flag) { if (flag) return "SELECT 4"; return "SELECT 5"; }
function scoped(flag) { if (flag) { const query = "SELECT 6"; return query; } return "SELECT 7"; }
function opaqueCondition() { if (opaquePredicate()) return "SELECT 8"; return "SELECT 9"; }
function arithmeticCondition() { if ({} + 1) return "SELECT 10"; return "SELECT 11"; }
write(truth()); // branch-true
write(falsity()); // branch-false
write(zero()); // branch-zero
write(nonzero()); // branch-nonzero
write(comparison()); // branch-comparison
write(reversed()); // branch-reversed
write(early()); // branch-early-return
write(same(unknownFlag)); // branch-same
write(different(unknownFlag)); // branch-different
write(scoped(unknownFlag)); // branch-local-binding
write(opaqueCondition()); // branch-opaque-condition
write(arithmeticCondition()); // branch-failed-condition
