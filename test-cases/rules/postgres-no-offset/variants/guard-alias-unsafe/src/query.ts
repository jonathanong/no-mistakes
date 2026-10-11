import { query } from "@example/db";
const gate = runtimeFlag;
const alias = gate;
const inverted = !alias;
let direct = "SELECT 1";
if (gate) direct += " OFFSET 1"; // findings: offset
if (alias) query(direct);
let negative = "SELECT 2";
if (gate) negative += " OFFSET 2"; // findings: offset
if (!inverted) query(negative);
let mutable = firstFlag;
const snapshot = mutable;
mutable = secondFlag;
let copied = "SELECT 3";
if (mutable) copied += " OFFSET 3"; // findings: offset
if (!snapshot) query(copied);
