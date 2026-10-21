import { tables as registry } from "./tables";

export const tables: Record<string, string> = {};

// Same-file write: unique producer is this module, so the registry rule abstains.
registry.users = "app.users";
