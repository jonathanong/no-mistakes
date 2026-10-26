import { read } from "@example/db";
import { forwarded } from "./patch-reexport.mjs";
import { changed } from "./patch-catalog-helper.mjs";
import { cycle } from "./patch-cycle-a.mjs";
read(forwarded()); // CLEAR_NAMED_REEXPORT
read(changed()); // ADVISE_MUTATED_EXPORT
read(cycle()); // ADVISE_CYCLIC_REEXPORT
