import { enqueueEmail } from "@app/jobs";
// These throws abort forEach before the later aggregate can observe its promises.
async function thrownAfterPush(xs) { const ps = []; xs.forEach(x => { ps.push(enqueueEmail(x)); throw error; }); await Promise.all(ps); }
async function conditionallyThrown(xs) { const ps = []; xs.forEach(x => { ps.push(enqueueEmail(x)); if (cond) throw error; }); await Promise.all(ps); }
async function thrownBeforeTry(xs) { const ps = []; xs.forEach(x => { ps.push(enqueueEmail(x)); throw error; }); try { run(); } finally { await Promise.all(ps); } }
