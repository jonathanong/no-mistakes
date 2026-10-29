// Reads, spreads, destructuring, and look-alikes of process.env are not writes.
const { A } = process.env;
const copy = { ...process.env };
const value = process.env.B;
const other = { process: { env: {} } };
other.process.env.C = "x";
function shadow(process) {
  process.env.D = "x";
  delete process.env.E;
}
const owner = {};
owner.env = 1;
delete owner.env;
delete other.process;
let local;
local = 1;
local += 2;
const items = [1].map((item) => item);
owner["k"] = 1;
delete getValue();
