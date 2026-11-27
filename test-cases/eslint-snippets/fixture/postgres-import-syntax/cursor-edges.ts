declare const flag: string;
declare const dyn: string;

const { runCursor } = require(flag);
runCursor(dyn);
const held = runCursor;

const cur = require(flag);
const member = cur.runCursor;

import { runCursor as open } from "@example/cursors";
import { sql as namedSql } from "@example/sql";

let loose = namedSql;
open(loose`SELECT 1`);

const { sql } = require(flag);
open(sql`SELECT 1`);

const tags = require(flag);
open(tags.sql`SELECT 1`);

import foreignNs = require("net");
foreignNs = flag;
open(foreignNs.sql`SELECT 1`);

import matched = require("@example/sql");
matched = flag;
const { sql: matchedSql } = matched;
open(matchedSql`SELECT 1`);

import fallback from "@example/sql";
open(fallback.sql`SELECT 1`);
