-- TABLE has no WHERE clause; rewriting it into SELECT would hide a parse error.
TABLE accounts WHERE id = $1;
