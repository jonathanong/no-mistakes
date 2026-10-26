import { enqueueEmail } from "@app/jobs";
import { consumeOptional, withPrototype, mutatePrototype } from "./cross-file-prototype-consumers";
const initial = () => enqueueEmail(1);
await consumeOptional({ job: initial, __proto__: { other() { this.job(); } } });
const forwarded = () => enqueueEmail(2);
await withPrototype({ job: forwarded });
const mutated = () => enqueueEmail(3);
await mutatePrototype({ job: mutated });
