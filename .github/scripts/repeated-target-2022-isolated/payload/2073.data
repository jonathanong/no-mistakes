-- Different schemas and table names must not collide during lexical recovery.
CREATE VIEW app.candidates AS TABLE other.orders UNION ALL TABLE app.customers UNION ALL TABLE app.orders;
