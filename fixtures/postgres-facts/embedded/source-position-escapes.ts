import { query } from '@data-stores/psql';
// Escapes and physical continuations must agree even for unpaired surrogates.
query("SELECT '\x41' FROM orders\n \
OFFSET 1");
query("SELECT '\u0041' FROM orders\n \
OFFSET 1");
query("SELECT '\u{1F600}' FROM orders\n \
OFFSET 1");
query("SELECT '\uD83D\uDE00' FROM orders\n \
OFFSET 1");
query("SELECT '\uD83D\u0041' FROM orders\n \
OFFSET 1");
query("SELECT '\uD83DX' FROM orders\n \
OFFSET 1");
query("SELECT '\0' FROM orders\n \
OFFSET 1");
