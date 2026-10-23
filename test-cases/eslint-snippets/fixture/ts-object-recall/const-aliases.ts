const renamed = original;
const asserted = original as Handler;
// Member reads, calls and destructures are outside this identifier-alias rule.
const property = db[`read`];
const called = original?.();
const { read } = db;
