import { enqueueEmail } from "@app/jobs";

function undrained(c) {
  const ps = [];
  ps.push(enqueueEmail("first"));
  if (c) ps.push(enqueueEmail("conditional"));
}

async function retainedBeforeConditional(c) {
  const ps = [];
  const p = enqueueEmail("not-always-pushed");
  if (c) ps.push(Promise.resolve(p));
  await Promise.all(ps);
}
