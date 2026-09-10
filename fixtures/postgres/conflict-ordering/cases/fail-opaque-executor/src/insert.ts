import { query, read, write } from "@data-stores/psql";

declare function assembleWriter(): string;

query(assembleWriter());
read(assembleWriter());
write(assembleWriter());
