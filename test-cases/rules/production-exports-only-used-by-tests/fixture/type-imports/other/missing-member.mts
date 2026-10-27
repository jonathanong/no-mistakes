// The module is known but exports no such member: still reported as unresolved.
import type * as Present from "../src/missing-member.mjs";
export type Gone = Present.Absent;
