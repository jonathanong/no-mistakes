import { read } from "@example/db";
import { assigned, shorthand, renamedWrite, arrayWrite, incremented, decremented, iterated, keyed, combined, shadowed } from "./writes";
import { evaluated } from "./evaluated";
import { evaluatedWrapped } from "./evaluated-wrapped";
assigned(read);
shorthand(read);
renamedWrite(read);
arrayWrite(read);
incremented(read);
decremented(read);
iterated(read);
keyed(read);
combined(read);
evaluated(read);
evaluatedWrapped(read);
shadowed(read);
// Repeated targets reuse the exporting source's request-scoped facts.
shadowed(read);
