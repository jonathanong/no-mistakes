import { query } from "@data-stores/psql";

declare function assembleWriter(): string;

query(assembleWriter());
