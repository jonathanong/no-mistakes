import sql from "sql-template-strings";
import { write } from "@app/db";

let arrayStatement = sql`/* array alias */ SELECT 1`;
const arrayAlias = arrayStatement;
const arrayIgnored = ([arrayStatement] = ["replacement"]);
write(arrayAlias); // known:array-target-keeps-alias
write(arrayStatement); // unanalyzable:array-target-rebound

let objectStatement = sql`/* object alias */ SELECT 1`;
const objectAlias = objectStatement;
const objectIgnored = ({ value: objectStatement } = { value: "replacement" });
write(objectAlias); // known:object-target-keeps-alias
write(objectStatement); // unanalyzable:object-target-rebound

let defaultStatement = sql`/* default alias */ SELECT 1`;
const defaultAlias = defaultStatement;
// Defaults and computed keys remain effects even when binding targets are omitted.
const defaultIgnored = ([defaultStatement = unknownMutation(defaultAlias)] = []);
write(defaultAlias); // unanalyzable:default-effect

let keyStatement = sql`/* computed key alias */ SELECT 1`;
const keyAlias = keyStatement;
const keyIgnored = ({ [unknownMutation(keyAlias)]: keyStatement } = {});
write(keyAlias); // unanalyzable:computed-key-effect

const memberStatement = sql`/* member target alias */ SELECT 1`;
const memberAlias = memberStatement;
const memberIgnored = ([memberStatement.text] = ["replacement"]);
write(memberAlias); // unanalyzable:member-target-effect

let restStatement = sql`/* rest alias */ SELECT 1`;
const restAlias = restStatement;
const restIgnored = ([...restStatement] = ["replacement"]);
write(restAlias); // known:rest-target-keeps-alias

let shorthandStatement = sql`/* shorthand alias */ SELECT 1`;
const shorthandAlias = shorthandStatement;
const shorthandIgnored = ({ shorthandStatement } = { shorthandStatement: "replacement" });
write(shorthandAlias); // known:shorthand-target-keeps-alias
