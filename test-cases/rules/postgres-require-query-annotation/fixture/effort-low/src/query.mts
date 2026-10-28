import { read, query, sql, sql as renamedTag } from "@bench/db";
import { importedSql, buildSql } from "./shared.mjs";
declare const body: string;
declare const flag: boolean;
declare const container: { statement: string };

query("/* direct */ SELECT 1"); // EXPECT_ACCEPTED
read(`/* template */ ${body}`); // EXPECT_ACCEPTED
sql("/* left literal */ " + body); // EXPECT_ACCEPTED
const statement = `/* one local const */ ${body}`;
query(statement); // EXPECT_ACCEPTED

read(helper("/* helper argument */ SELECT 1")); // EXPECT_REPORT_HELPER
read(sql`/* configured tag */ SELECT 1`); // EXPECT_ACCEPTED_TAG
read(flag ? "/* conditional a */ SELECT 1" : "/* conditional b */ SELECT 2"); // EXPECT_ACCEPTED_CONDITIONAL (both runtime arms have named literal heads)
read(importedSql); // EXPECT_ACCEPTED_IMPORT (one source-owned immutable literal)
read(buildSql()); // EXPECT_REPORT_CALL_SITE (helper-only suppression does not apply)
let mutable = "/* mutable */ SELECT 1";
read(mutable); // EXPECT_REPORT_MUTABLE
const alias = statement;
read(alias); // EXPECT_REPORT_ALIAS
read(container.statement); // EXPECT_REPORT_MEMBER
read("BEGIN"); // EXPECT_ACCEPTED_TRANSACTION
// no-mistakes-disable-next-line postgres-require-query-annotation: call-site suppression
read(body); // EXPECT_LINE_SUPPRESSED
read(body); // no-mistakes-disable-line postgres-require-query-annotation: same-line suppression

// Low effort judges literal heads independently of tag identity or configuration.
import { sql as subpathTag } from "@bench/db/sql";
import { sql as unrelatedTag } from "@other/db";
import builtinTag from "sql-template-strings";
read(renamedTag`/* configured alias */ SELECT ${body}`); // EXPECT_ACCEPTED_TAG
read(subpathTag`/* configured subpath */ SELECT ${body}`); // EXPECT_ACCEPTED_TAG
const taggedStatement = renamedTag`/* one tagged const */ SELECT ${body}`;
read(taggedStatement); // EXPECT_ACCEPTED_TAG
const subpathStatement = subpathTag`/* subpath const */ SELECT ${body}`;
read(subpathStatement); // EXPECT_ACCEPTED_TAG
read(unrelatedTag`/* unrelated */ SELECT 1`); // EXPECT_ACCEPTED_TAG
read(builtinTag`/* default builtin is not configured */ SELECT 1`); // EXPECT_ACCEPTED_TAG
function tagShadow(renamedTag: unknown) { read(renamedTag`/* shadow */ SELECT 1`); } // EXPECT_ACCEPTED_TAG
{ const renamedTag = helper; read(renamedTag`/* block shadow */ SELECT 1`); } // EXPECT_ACCEPTED_TAG
read(sql`/* open ${body} */ SELECT 1`); // EXPECT_REPORT_OPEN_TAG
read(sql`/*  */ SELECT 1`); // EXPECT_REPORT_EMPTY_TAG EXPECT_MISSING
const aliasSource = renamedTag`/* alias source */ SELECT ${body}`;
const taggedAlias = aliasSource;
read(taggedAlias); // EXPECT_REPORT_TAG_ALIAS
const openTag = sql`/* incomplete ${body} */ SELECT 1`;
read(openTag); // EXPECT_REPORT_OPEN_CONST_TAG
const wrongTag = unrelatedTag`/* unconfigured const */ SELECT 1`;
read(wrongTag); // EXPECT_ACCEPTED_TAG
const changedTag = renamedTag`/* changed builder */ SELECT 1`;
changedTag.text = "SELECT unsafe";
read(changedTag); // EXPECT_REPORT_MUTATED_TAG
