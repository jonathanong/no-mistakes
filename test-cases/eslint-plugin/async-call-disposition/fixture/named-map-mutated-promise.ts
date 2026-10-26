import { enqueueEmail } from "@app/jobs";

const callback = (x) => enqueueEmail(x);
Promise.all = substitute;
await Promise.all(items.map(callback));
