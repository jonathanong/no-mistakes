import { query } from "@example/db";
query(first ? "SELECT '\u{1F600}\uD83D\uDE00\x41\u0042\u{43}é' OFFSET 101" : "SELECT 'ok' OFFSET 102");
query(second ? "SELECT '\uD800\u{DFFF}' OFFSET 103" : "SELECT 'ok' OFFSET 104");
query(third ? `SELECT 'first'
 OFFSET 105` : `SELECT 'second'\
 OFFSET 106`);
query(fourth ? String.raw`SELECT '\u{41}\n' OFFSET 107` : String.raw`SELECT '\x42' OFFSET 108`);
