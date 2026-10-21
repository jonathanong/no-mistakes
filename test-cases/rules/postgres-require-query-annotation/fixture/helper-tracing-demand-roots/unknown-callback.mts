// Opaque external calls still invoke callbacks from reachable prepared imports.
import { schedule } from "missing-external";
import { execute } from "./executor.mjs";
schedule(() => execute("SELECT callback"));
