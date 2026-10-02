// Legacy octal is legal in this non-strict CommonJS source.
db.query("SELECT '\400' FROM orders \
OFFSET 1");
db.query("SELECT '\377 \777' FROM orders \
OFFSET 0");
