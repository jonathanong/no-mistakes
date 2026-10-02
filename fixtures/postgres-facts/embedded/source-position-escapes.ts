import { query } from '@data-stores/psql';
// Every decoded escape stays on the physical call line, including surrogate pairs.
query("SELECT '\x41' FROM orders\nOFFSET 1");
query("SELECT '\u0041' FROM orders\nOFFSET 1");
query("SELECT '\u{1F600}' FROM orders\nOFFSET 1");
query("SELECT '\uD83D\uDE00' FROM orders\nOFFSET 1");
query("SELECT '\uD83D\u0041' FROM orders\nOFFSET 1");
query("SELECT '\uD83DX' FROM orders\nOFFSET 1");
query("SELECT '\0' FROM orders\nOFFSET 1");
