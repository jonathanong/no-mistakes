import { primary as receiver } from "@vendor/database";
import { queryHelper } from "./helpers.mjs";
for (const item of ["a", "b"]) {
  // The original transaction and the next iteration's helper share this site.
  receiver.transaction(item);
  receiver.transaction = queryHelper;
}
