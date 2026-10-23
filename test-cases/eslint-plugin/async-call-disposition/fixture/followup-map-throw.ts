import { enqueueEmail } from "@app/jobs";
// A later mapper iteration can abort before the array is assigned or drained.
async function thrownInMapper(xs) { const ps = xs.map(x => { if (x.bad) throw error; return enqueueEmail(x); }); await Promise.all(ps); }
async function thrownMapperFinally(xs) { const ps = xs.map(x => { if (x.bad) throw error; return enqueueEmail(x); }); try { run(); } finally { await Promise.all(ps); } }
