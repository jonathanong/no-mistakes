import { query } from "@bench/db";
import { registry, runViaLazy } from "./shared/registry.mjs";
import { runImpure } from "./shared/impure.mjs";

export function entry0() {
  registry.executeLeaf("/* entry0 */ SELECT 0", (s) => query(s));
  return runViaLazy("/* entry0_lazy */ SELECT 0", (s) => query(s));
}

export function entry1() {
  registry.executeLeaf("/* entry1 */ SELECT 1", (s) => query(s));
  return runViaLazy("/* entry1_lazy */ SELECT 1", (s) => query(s));
}

export function entry2() {
  registry.executeLeaf("/* entry2 */ SELECT 2", (s) => query(s));
  return runViaLazy("/* entry2_lazy */ SELECT 2", (s) => query(s));
}

export function entry3() {
  registry.executeLeaf("/* entry3 */ SELECT 3", (s) => query(s));
  return runViaLazy("/* entry3_lazy */ SELECT 3", (s) => query(s));
}

export function entry4() {
  registry.executeLeaf("/* entry4 */ SELECT 4", (s) => query(s));
  return runViaLazy("/* entry4_lazy */ SELECT 4", (s) => query(s));
}

export function entry5() {
  registry.executeLeaf("/* entry5 */ SELECT 5", (s) => query(s));
  return runViaLazy("/* entry5_lazy */ SELECT 5", (s) => query(s));
}

export function entry6() {
  registry.executeLeaf("/* entry6 */ SELECT 6", (s) => query(s));
  return runViaLazy("/* entry6_lazy */ SELECT 6", (s) => query(s));
}

export function entry7() {
  registry.executeLeaf("/* entry7 */ SELECT 7", (s) => query(s));
  runViaLazy("SELECT id FROM missing_annotation", (s) => query(s));
  return runImpure("/* entry7_impure */ SELECT 7");
}
