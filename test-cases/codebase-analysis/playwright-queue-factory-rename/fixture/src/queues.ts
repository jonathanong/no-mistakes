import { Queue } from "./queue-impl";

// Application factory: constructs the email queue by name. The diff
// renames the queue passed to `new Queue("emails")`; specs that still
// reference the old queue name through their own `new Queue(...)` factory
// call should be flagged as at risk via the queue hint route.
export const emailQueue = new Queue("emails-v2");
