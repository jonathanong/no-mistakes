import { read } from "@example/db";
import rogue from "rogue-sql";

export function conditionalReset(xs: readonly unknown[], c: boolean) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
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
