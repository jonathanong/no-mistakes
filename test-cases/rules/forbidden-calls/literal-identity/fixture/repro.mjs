// Keep escaped surrogate, backslash-u, replacement, and astral identities distinct.
inspect(
  "\uD800", "\uDC00", "\\ud800", "\uFFFD", "\uFFFD\uD800", "\uD800\uDC00",
  `\uD800`, `\uDC00`, `\\ud800`, `\uFFFD`, `\uFFFD\uD800`, `\uD800\uDC00`,
  {
    "\uD800": "0",
    "\uDC00": "1",
    "\\ud800": "2",
    "\uFFFD": "3",
    "\uFFFD\uD800": "4",
    "\uD800\uDC00": "5",
  },
);
