import { query } from "@db/a";
// Keep physical CRLF inside this multiline SQL template.
query(`SELECT id
FROM low_control
OFFSET 8`);
