import SQL from 'sql-template-strings';
import { read } from '@example/db';
function trueControl() { return true ? SQL`SELECT 1` : unknownCall(); }
function falseControl() { return false ? unknownCall() : SQL`SELECT 2`; }
function trueFailure() { return true ? unknownCall() : SQL`SELECT 1`; }
function falseFailure() { return false ? SQL`SELECT 1` : unknownCall(); }
function uncertainYes() { return 1 ? unknownCall() : SQL`SELECT 1`; }
function uncertainNo() { return 1 ? SQL`SELECT 1` : unknownCall(); }
function conditionFailure() { return unknownCall() ? SQL`SELECT 1` : SQL`SELECT 2`; }
function memberFailure() { return unknownCall().text; }
function computedObjectFailure() { return unknownCall()['text']; }
function computedKeyFailure() { const object = {text: SQL`SELECT 1`}; return object[unknownCall()]; }
function appendObjectFailure() { return unknownCall().append(' FROM accounts'); }
function appendSuffixFailure() { const query = SQL`SELECT 1`; return query.append(unknownCall()); }
read(trueControl());
read(falseControl());
read(trueFailure());
read(falseFailure());
read(conditionFailure());
read(memberFailure());
read(computedObjectFailure());
read(computedKeyFailure());
read(appendObjectFailure());
read(appendSuffixFailure());
