import { createExecutor } from "@example/db";
import type { Executor } from "@example/db";
const execute = createExecutor(); // unsupported factory reference
execute("SELECT * FROM public.users");
function parameter(db: Executor) { // Erased types do not create unknown SQL uses.
  db.query("SELECT * FROM public.users");
}
