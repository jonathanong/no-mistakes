// Keep U+FFFD distinct from lone surrogates: report identities predate WTF-8.
const values = [
  "plain", "café 世界", "�", "\uFFFD", "😀", "\uD83D\uDE00",
  "\uD800", "\uDC00", "\uDFFF", "\u{D800}",
  "�\uD800�", "\uFFFD\uD800\uFFFD", "😀\uD800x\uDC00",
  `plain`, `café 世界`, `�`, `\uFFFD`, `😀`, `\uD83D\uDE00`,
  `\uD800`, `\uDC00`, `�\uD800�`, `\uFFFD\uD800\uFFFD`,
  `\uD800${value}�`, `�${value}\uD800`,
  String.raw`\u{invalid}`,
  // Braced escapes and line continuations did not pair in Oxc 0.153.
  "\u{D800}\u{DC00}", "\uD800\u{DC00}", "\u{D800}\uDC00",
  "�\u{000D800}\u{000DC00}�", "\u{D800}\uD83D\uDE00",
  "\uD800\
\uDC00",
  `\u{D800}\u{DC00}`, `\uD800\u{DC00}`, `\u{D800}\uDC00`,
  `�\u{D800}\u{DC00}�`, `\uD800\
\uDC00`,
  "\x61\n\u{D800}\u{DC00}\t\'\\", "\u{1F600}",
  // Escaped backslashes are text, not Unicode escapes.
  "\\u{D800}\u{DC00}",

];
