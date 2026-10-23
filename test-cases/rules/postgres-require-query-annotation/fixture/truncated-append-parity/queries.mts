import sql from "sql-template-strings";
import { read } from "@bench/db";

const query = sql`SELECT (`;
if (c00) query.append("1)");
if (c01) query.append("1)");
if (c02) query.append("1)");
if (c03) query.append("1)");
if (c04) query.append("1)");
if (c05) query.append("1)");
if (c06) query.append("1)");
if (c07) query.append("1)");
if (c08) query.append("1)");
if (c09) query.append("1)");
if (c10) query.append("1)");
if (c11) query.append("1)");
if (c12) query.append("1)");
if (c13) query.append("1)");
if (c14) query.append("1)");
if (c15) query.append("1)");
if (c16) query.append("1)");
if (c17) query.append("1)");
if (c18) query.append("1)");
if (c19) query.append("1)");
read(query);
// no-mistakes-disable-next-line postgres-require-query-annotation
read(query);
