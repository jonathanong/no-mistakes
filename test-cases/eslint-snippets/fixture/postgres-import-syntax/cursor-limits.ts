import { runCursor } from "@example/cursors";
import * as tags from "@example/sql/tags";

runCursor(tags.sql`/* ok */ SELECT 1`);

declare const flag: string;
declare const dyn: string;

import { runCursor as other } from "net";
other = flag;
other(dyn);

require("net").runCursor(dyn);

import db = require("net");
db = flag;
db.runCursor(dyn);
