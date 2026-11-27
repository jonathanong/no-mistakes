import { openCursor } from "@example/cursors";
import sql from "sql-template-strings";

type Row = { id: number };

(openCursor<Row>)("/* ok */ SELECT 1");
openCursor((sql<Row>)`/* ok */ SELECT 1`);
