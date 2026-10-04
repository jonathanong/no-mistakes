import { query, read, write } from "@example/db";

declare function assembleWriter(): string;

query(assembleWriter());
read(assembleWriter());
write(assembleWriter());
