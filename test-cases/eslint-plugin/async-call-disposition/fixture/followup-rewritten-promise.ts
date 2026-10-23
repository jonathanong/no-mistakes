import { enqueueEmail } from "@app/jobs";
Promise.resolve = unknown;
function rewritten() { return Promise.resolve(enqueueEmail("rewritten")); }
