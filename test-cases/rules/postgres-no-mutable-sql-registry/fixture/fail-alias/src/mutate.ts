import { tables as registry } from "./tables";
import { tables as also } from "./tables";

// Two aliases of the same export: producers sort/dedup to one writer triple.
registry.users = "app.users";
also.orders = "app.orders";
