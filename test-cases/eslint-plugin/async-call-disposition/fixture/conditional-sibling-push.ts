import { enqueueEmail } from "@app/jobs";

// A promise created as a push argument is retained whenever its branch runs.
async function guardedPush() { const ps = []; if (cond) ps.push(enqueueEmail("conditional")); await Promise.all(ps); }

async function drained(c) {
  const ps = [];
  ps.push(enqueueEmail("first"));
  if (c) ps.push(enqueueEmail("conditional"));
  await Promise.all(ps);
}

async function braced(c) {
  const ps = [];
  ps.push(enqueueEmail("first"));
  if (c) {
    ps.push(enqueueEmail("conditional"));
  }
  await Promise.all(ps);
}
