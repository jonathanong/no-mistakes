import defaultJob from "./syntax-default";
interface Indexed { [key: string]: (() => Promise<void>); }
const [head, , ...tail] = [0, 1, 2];
let assigned, remainder;
({ value: assigned = 1, ...remainder } = { value: head, tail });
({ assigned = 1 } = { assigned: 2 });
const sequence = (0, 1);
const asserted = assigned!;
function* values() { yield sequence; yield* tail; }
export async function consume(callback: () => Promise<void>, initial = 0) {
  var counter = initial;
  for (var i = 0; i < 1; i++) { if (i) continue; break; }
  while (counter) { counter--; }
  do { counter++; } while (false);
  try { throw asserted; } catch (error) { void error; } finally { void values; }
  await callback();
}
