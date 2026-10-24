import { read } from "@example/db";
import rogue from "rogue-sql";

export function conditionalReset(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    // Calls cannot supply a pure condition key; these empty arms leave q untouched.
    if (namedSql()) {}
    if (typeof namedSql()) {}
    if (namedSql() === "x") {}
    if ("x" === namedSql()) {}
    if (namedSql() && x) {}
    if (x && namedSql()) {}
    read(q); // missing: the false branch retains the literal.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
  }
}

export function conditionalBreakAfter(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the false branch reaches the backedge.
    q = "SELECT 2";
    if (c) break;
  }
}

export function conditionalBreakBefore(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the false branch reaches the assignment.
    if (c) break;
    q = "SELECT 2";
  }
}

export function conditionalContinueAfter(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: continue carries the literal before the reset.
    q = "SELECT 2";
    if (c) continue;
    q = "/* reset */ SELECT 3";
  }
}

export function resetOnExitPath(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the reset exits, and the false branch carries.
    q = "SELECT 2";
    if (c) {
      q = "/* reset */ SELECT 3";
      break;
    }
  }
}

export function allPathsReset(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    else q = "/* reset */ SELECT 4";
  }
}

export function allPathsExit(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (c) break;
    else break;
  }
}

export function allPathsResetBeforeRead(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    if (c) q = "/* reset */ SELECT 3";
    else q = "/* reset */ SELECT 4";
    read(q);
    q = "SELECT 2";
  }
}

export function conditionalContinueBeforeLiteral(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the false branch reaches the literal.
    if (c) continue;
    q = "SELECT 2";
  }
}

// A terminal statement on the fallthrough path must not erase a prior
// conditional continue backedge.
export function continueThenBreak(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: continue carries before the terminal break.
    q = "SELECT 2";
    if (c) continue;
    break;
  }
}

export function continueThenReturn(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: continue carries before the terminal return.
    q = "SELECT 2";
    if (c) continue;
    return;
  }
}

export function continueThenThrow(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: continue carries before the terminal throw.
    q = "SELECT 2";
    if (c) continue;
    throw new Error("stop");
  }
}

export function breakOrReturn(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (c) break;
    return;
  }
}

// A literal true condition has no fallthrough reset-free path.
export function alwaysReset(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (true) q = "/* reset */ SELECT 3";
  }
}

export function neverReset(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: a literal false reset cannot cover the backedge.
    q = "SELECT 2";
    if (false) q = "/* reset */ SELECT 3";
  }
}

export function alwaysExit(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (true) break;
  }
}

export function neverExit(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: a literal false break cannot cover the backedge.
    q = "SELECT 2";
    if (false) break;
  }
}

// Complementary tests of the same unchanged condition cover every path.
export function complementaryResets(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    if (!c) q = "/* reset */ SELECT 4";
  }
}

// Repeated guards before the final literal cannot remove its backedge.
export function overwrittenComplementaryResets(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the final literal overwrites both guarded resets.
    if (c) q = "/* reset */ SELECT 3";
    if (!c) q = "/* reset */ SELECT 4";
    q = "SELECT 2";
  }
}

// The trusted literal reaches continue even though an untrusted tag is later.
export function mixedTagPaths(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: an untrusted fallthrough tag cannot hide the literal path.
    q = "SELECT 2";
    if (c) continue;
    q = rogue`SELECT 3`;
  }
}

// A harmless earlier branch with the same guard cannot cover a later reset.
export function harmlessRepeatedGuard(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    if (c) console.log(x);
    read(q); // missing: the false branch still carries the literal.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
  }
}

export function namedConstReset(xs: readonly unknown[], c: boolean) {
  const named = "/* reset */ SELECT 3";
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: an indirect reset only covers the true branch; low-unknown.
    q = "SELECT 2";
    if (c) q = named;
  }
}

function namedSql() { return "/* reset */ SELECT 3"; }

export function helperReturnReset(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: a helper reset only covers the true branch; low-unknown.
    q = "SELECT 2";
    if (c) q = namedSql();
  }
}

export function binaryTestReset(xs: readonly unknown[], count: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: a pure comparison still leaves the other path live.
    q = "SELECT 2";
    if (count > 0) q = "/* reset */ SELECT 3";
  }
}

export function negatedBinaryReset(xs: readonly unknown[], count: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the negated comparison has a live other path.
    q = "SELECT 2";
    if (!(count > 0)) q = "/* reset */ SELECT 3";
  }
}

// Nested tests of the unchanged condition cover every real path.
export function nestedRepeatedReset(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (c) {
      if (c) q = "/* reset */ SELECT 3";
    } else q = "/* reset */ SELECT 4";
  }
}

// Opposite tests of the same pure comparison reset every real path.
export function complementaryComparisonResets(xs: readonly unknown[], count: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (count > 0) q = "/* reset */ SELECT 3";
    if (!(count > 0)) q = "/* reset */ SELECT 4";
  }
}

// Pure logical tests with primitive operands are correlated as a whole.
export function pureLogicalResets(
  xs: readonly unknown[], count: number, active: boolean, big: bigint, maybe: unknown,
) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (count > 0 && active === true && typeof count === "number" && maybe !== null && big === 1n)
      q = "/* reset */ SELECT 3";
    if (!(count > 0 && active === true && typeof count === "number" && maybe !== null && big === 1n))
      q = "/* reset */ SELECT 4";
  }
}

// A long pure test must retain its key; a size cutoff would invent a path.
export function longComparisonResets(xs: readonly unknown[], count: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    if (count + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15 + 16 + 17 > 0)
      q = "/* reset */ SELECT 3";
    if (!(count + 1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10 + 11 + 12 + 13 + 14 + 15 + 16 + 17 > 0))
      q = "/* reset */ SELECT 4";
  }
}

// Same-polarity comparisons leave a live path even when the input changes.
export function repeatedIncrementGuard(xs: readonly unknown[], i: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: both tests can be false after i increments.
    q = "SELECT 2";
    if (i > 0) q = "/* reset */ SELECT 3";
    i++;
    if (i > 0) q = "/* reset */ SELECT 4";
  }
}

export function repeatedLogicalGuard(xs: readonly unknown[], a: boolean, b: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the same false logical guard leaves the literal live.
    q = "SELECT 2";
    if (a && b) q = "/* reset */ SELECT 3";
    if (a && b) q = "/* reset */ SELECT 4";
  }
}

export function repeatedComparisonGuard(xs: readonly unknown[], n: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: n <= 0 makes both same-polarity tests false.
    q = "SELECT 2";
    if (n > 0) q = "/* reset */ SELECT 3";
    if (n > 0) q = "/* reset */ SELECT 4";
  }
}

export function repeatedEqualityAfterWrite(xs: readonly unknown[], n: number) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: n can miss zero before and after the write.
    q = "SELECT 2";
    if (n === 0) q = "/* reset */ SELECT 3";
    n = n + 1;
    if (n === 0) q = "/* reset */ SELECT 4";
  }
}

export function changedInsideFirstBranch(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: c=false skips both same-polarity resets.
    q = "SELECT 2";
    if (c) { c = false; q = "/* reset */ SELECT 3"; }
    if (c) q = "/* reset */ SELECT 4";
  }
}

export function changedBetweenGuards(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: c=false skips both resets.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    c = false;
    if (c) q = "/* reset */ SELECT 4";
  }
}

export function sameGuardContinueBreak(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: c=false reaches the backedge; c=true continues.
    q = "SELECT 2";
    if (c) continue;
    if (c) break;
  }
}

// Opposite tests alone do not cover a partial reset inside the first arm.
export function partialComplementReset(xs: readonly unknown[], c: boolean, d: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: c=true and d=false retains the literal.
    q = "SELECT 2";
    if (c) { if (d) q = "/* reset */ SELECT 3"; }
    if (!c) q = "/* reset */ SELECT 4";
  }
}

export function complementContinueBreak(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: c=true continues with the literal.
    q = "SELECT 2";
    if (c) continue;
    if (!c) break;
  }
}

// A write between opposite tests makes their outcomes independent.
export function changedBetweenComplements(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: initial c=false skips both resets after c becomes true.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    c = true;
    if (!c) q = "/* reset */ SELECT 4";
  }
}

// The else arm changes c before the opposite test without resetting q.
export function changedInsideComplementArm(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: initial c=false writes true, then skips the second reset.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    else c = true;
    if (!c) q = "/* reset */ SELECT 4";
  }
}

// A var redeclaration writes the same binding without a reference node.
export function redeclaredComplement(xs: readonly unknown[]) {
  var c = false;
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the redeclaration makes both opposite tests false.
    q = "SELECT 2";
    if (c) q = "/* reset */ SELECT 3";
    var c = true;
    if (!c) q = "/* reset */ SELECT 4";
  }
}

// The header update prevents a fixed i projection across iterations.
export function updatedHeaderCondition(start: number) {
  let q = "/* named */ SELECT 1";
  for (let i = start; i >= 0; i--) {
    read(q); // missing: deliberate conservative over-report; header writes i.
    q = "SELECT 2";
    if (i > 0) q = "/* reset */ SELECT 3";
    if (!(i > 0)) q = "/* reset */ SELECT 4";
  }
}
