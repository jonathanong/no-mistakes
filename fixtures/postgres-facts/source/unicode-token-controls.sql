-- U next to an ampersand is an operator operand unless followed by a quoted string.
SELECT u&identifier, u & 'separated', "U"&'quoted identifier',
       json_doc ? 'key', point_a <-> point_b, B'101', 1_000,
       U&'d\0061t\+000061';
