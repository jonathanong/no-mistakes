import { enqueueEmail } from "@app/jobs";
import { awaitReportedEnqueueAdmission as consume, other } from "@app/consumer";
async function run() {
  await consume(() => { return enqueueEmail(1); });
  consume(() => enqueueEmail(2)); // Configured callback returns are a trusted contract.
  consume(() => { const p = enqueueEmail(3); return p; });
  consume(() => enqueueEmail(4).finally(cleanup));
  consume(() => Promise.all([enqueueEmail(5)]));
  consume.call(null, () => enqueueEmail(6));
  consume.apply(null, [() => enqueueEmail(7)]); // Preserve existing array-contained callback behavior.
  const alias = consume;
  alias(() => enqueueEmail(8));
  consume(...args, () => enqueueEmail(9));
  other(() => enqueueEmail(10));
  consume(null, () => enqueueEmail(11));
  consume(() => { enqueueEmail(12); });
  function shadow(consume) { consume(() => enqueueEmail(13)); }
  let reassigned = consume;
  reassigned = other;
  reassigned(() => enqueueEmail(14));
  consume(() => { return Promise.all([enqueueEmail(15)]); });
  consume(() => { return (() => enqueueEmail(16))(); });
  consume.apply(null, [() => enqueueEmail(17), () => enqueueEmail(18)]);
}

consume.call.call(unknown, null, () => enqueueEmail(19)); // The explicit receiver invokes unknown.
