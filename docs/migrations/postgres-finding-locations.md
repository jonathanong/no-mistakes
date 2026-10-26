# PostgreSQL findings belong to their own statements

PostgreSQL findings now use the accepted statement's AST and existing source
tokens. A later statement cannot inherit a line from an earlier statement that
mentions the same table, column, or constraint. Comments and string literals
also cannot supply that location.

For example, these foreign keys report on lines 1 and 2 respectively:

```sql
ALTER TABLE "Attribute" ADD CONSTRAINT "Attribute_personId_fkey" FOREIGN KEY ("personId") REFERENCES "Person"("id");
ALTER TABLE "Session" ADD CONSTRAINT "Session_personId_fkey" FOREIGN KEY ("personId") REFERENCES "Person"("id");
```

The correction also covers ADD COLUMN, ADD/VALIDATE constraints, statement
policies, indexes, INSERTs, SELECTs, and triggers. Unsupported earlier statements
cannot shift an accepted statement's index or INSERT location.

Existing declaration anchors remain supported: inline CREATE TABLE foreign-key
allow directives stay beside their declaration, and SELECT findings retain their
own FROM relation line. Statements without a relation use their own keyword.
Place line suppressions and FK allow directives at the statement or declaration
the finding identifies. A later statement's suppression does not exempt an
earlier sibling.

CLI, CommonJS, ESM, and prepared project reports share these locations. Worker
count does not affect finding order or file/line output. Parsing decisions and
the unanalyzable policy are unchanged.
