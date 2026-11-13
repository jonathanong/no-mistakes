import sql from "sql-template-strings";
import {read} from "@bench/db";
import {literal, trusted, untrusted, interpolated, template, noAppend, wrongReturn, mutableAlias, optional, wrongBase, extraEffect, eight, nine} from "./returned-sources.mjs";
const result_literal = literal(sql`/* literal */ SELECT 1`);
read(result_literal.text, result_literal.values); // low: true
const result_trusted = trusted(sql`/* trusted */ SELECT 1`);
read(result_trusted.text, result_trusted.values); // low: true
const result_untrusted = untrusted(sql`/* untrusted */ SELECT 1`);
read(result_untrusted.text, result_untrusted.values); // low: false
const result_interpolated = interpolated(sql`/* interpolated */ SELECT 1`);
read(result_interpolated.text, result_interpolated.values); // low: false
const result_template = template(sql`/* template */ SELECT 1`);
read(result_template.text, result_template.values); // low: false
const result_noAppend = noAppend(sql`/* noAppend */ SELECT 1`);
read(result_noAppend.text, result_noAppend.values); // low: false
const result_wrongReturn = wrongReturn(sql`/* wrongReturn */ SELECT 1`);
read(result_wrongReturn.text, result_wrongReturn.values); // low: false
const result_mutableAlias = mutableAlias(sql`/* mutableAlias */ SELECT 1`);
read(result_mutableAlias.text, result_mutableAlias.values); // low: false
const result_optional = optional(sql`/* optional */ SELECT 1`);
read(result_optional.text, result_optional.values); // low: false
const result_wrongBase = wrongBase(sql`/* wrongBase */ SELECT 1`);
read(result_wrongBase.text, result_wrongBase.values); // low: false
const result_extraEffect = extraEffect(sql`/* extraEffect */ SELECT 1`);
read(result_extraEffect.text, result_extraEffect.values); // low: false
const result_eight = eight(sql`/* eight */ SELECT 1`);
read(result_eight.text, result_eight.values); // low: true
const result_nine = nine(sql`/* nine */ SELECT 1`);
read(result_nine.text, result_nine.values); // low: false
const joined = choice ? literal(sql`/* left */ SELECT 1`) : trusted(sql`/* right */ SELECT 1`);
read(joined.text, joined.values); // low: true
const rejected = choice ? literal(sql`/* left */ SELECT 1`) : untrusted(sql`/* right */ SELECT 1`);
read(rejected.text, rejected.values); // low: false
import {literal as absent} from "./not-prepared.mjs";
const missing = absent(sql`/* missing */ SELECT 1`);
read(missing.text, missing.values); // low: false
import {literal as broken} from "./broken.mjs";
const failed = broken(sql`/* failed */ SELECT 1`);
read(failed.text, failed.values); // low: false
