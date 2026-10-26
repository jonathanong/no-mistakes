import { query } from "@example/db";
declare function unrelated(text: string): void;
// Only the configured call's argument may be synthesized as SQL input.
unrelated("not SQL");
const unused = "unrelated initializer";
query("SELECT * FROM public.users");
