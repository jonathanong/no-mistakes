// Equal report text can carry different lexer flags; do not infer flags from markers.
const values = ["\uD800", "�d800", "\u{D800}\u{DC00}", "\uD83D\uDE00", "😀"];
