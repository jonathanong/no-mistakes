-- STDIN here is prose, so normalization must preserve the source.
/* outer
 /* nested
  */ end */
SELECT E'escaped\
line', 'doubled''quote', "doubled""identifier", $tag$dollar
quote$tag$, foo$bar, _foo$tag$, café$tag$, $1;
