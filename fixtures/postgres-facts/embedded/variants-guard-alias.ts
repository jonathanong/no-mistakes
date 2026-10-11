import { query } from "@example/db";
const gate = runtimeFlag;
const alias = gate;
let direct = "SELECT 1";
if (gate) direct += " OFFSET 1";
if (!alias) query(direct); // variants: SELECT 1
if (alias) query(direct); // variants: SELECT 1 OFFSET 1
const chained = (alias as boolean);
const inverted = !chained;
let negative = "SELECT 2";
if (gate) negative += " OFFSET 2";
if (inverted) query(negative); // variants: SELECT 2
if (!inverted) query(negative); // variants: SELECT 2 OFFSET 2
const doubled = !!(gate as boolean);
let positive = "SELECT 3";
if (gate) positive += " OFFSET 3";
if (!doubled) query(positive); // variants: SELECT 3
if (doubled) query(positive); // variants: SELECT 3 OFFSET 3
const restored = !inverted;
let restoredText = "SELECT 4";
if (gate) restoredText += " OFFSET 4";
if (!restored) query(restoredText); // variants: SELECT 4
if (restored) query(restoredText); // variants: SELECT 4 OFFSET 4
{
  const gate = otherFlag;
  const alias = gate;
  let shadowed = "SELECT 5";
  if (gate) shadowed += " OFFSET 5";
  if (!alias) query(shadowed); // variants: SELECT 5
  if (alias) query(shadowed); // variants: SELECT 5 OFFSET 5
}
let mutable = firstFlag;
const snapshot = mutable;
mutable = secondFlag;
let copiedText = "SELECT 6";
if (mutable) copiedText += " OFFSET 6";
if (!snapshot) query(copiedText); // variants: SELECT 6 | SELECT 6 OFFSET 6
function parameterSnapshot(value: boolean) {
  const copied = value;
  value = thirdFlag;
  let text = "SELECT 7";
  if (value) text += " OFFSET 7";
  if (!copied) query(text); // variants: SELECT 7 | SELECT 7 OFFSET 7
}
