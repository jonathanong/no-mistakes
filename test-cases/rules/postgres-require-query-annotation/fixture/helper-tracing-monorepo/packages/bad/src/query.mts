import { write } from "@app/db";
import { statement } from "@local/query";
await write(statement());
