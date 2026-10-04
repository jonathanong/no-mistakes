-- Final-arm inclusivity and per-key sort directions preserve a complete cursor boundary.
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b >= $2) ORDER BY a,b LIMIT $3;
SELECT * FROM accounts WHERE a < $1 OR (a = $1 AND b <= $2) ORDER BY a,b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a < $1 OR (a = $1 AND b > $2) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a < $1 OR (a = $1 AND b >= $2) ORDER BY a DESC,b ASC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b <= $2) ORDER BY a DESC,b ASC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) OR (a = $1 AND b = $2 AND c >= $3) ORDER BY a ASC,b DESC,c ASC LIMIT $4;
SELECT * FROM accounts WHERE (a = $1 AND b <= $2) OR a > $1 ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE $1 < a OR ($1 = a AND $2 >= b) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 ORDER BY a DESC LIMIT $2;
-- Inclusive earlier arms swallow later prefixes; wrong direction, identity, and ordering stay selective.
SELECT * FROM accounts WHERE a >= $1 OR (a = $1 AND b > $2) ORDER BY a,b LIMIT $3;
SELECT * FROM accounts WHERE a <= $1 OR (a = $1 AND b < $2) ORDER BY a,b LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b > $2) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $9 AND b < $2) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) ORDER BY b DESC,a ASC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) ORDER BY a USING >,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < $2) OR (a = $1 AND b = $2 AND c < $3) ORDER BY a ASC,b DESC,c ASC LIMIT $4;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b < 2) ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR b < $2 ORDER BY a ASC,b DESC LIMIT $3;
SELECT * FROM accounts WHERE a > $1 OR (a = $1 AND b <= $2) OR (a = $1 AND b = $2 AND c > $3) ORDER BY a ASC,b DESC,c ASC LIMIT $4;
