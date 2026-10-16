-- Recognized static LOCK utility facts coexist with the safely attributed nested constraint.
CREATE INDEX before_constraint_idx ON "Mý"."Chïld"("Parent Id");
DO $outer$ BEGIN
 IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'FK雪') THEN
  LOCK TABLE "Mý"."Parent" IN SHARE ROW EXCLUSIVE MODE;
  LOCK TABLE "Mý"."Chïld" IN SHARE ROW EXCLUSIVE MODE;
  IF CASE WHEN true THEN NOT EXISTS (SELECT 1 WHERE 'END IF; ELSE' = 'text') ELSE false END THEN
   /* END IF; is a comment, never a statement boundary. */
   ALTER TABLE "Mý"."Chïld" ADD CONSTRAINT "FK雪"
    FOREIGN KEY ("Parent Id") REFERENCES "Mý"."Parent"("Id") NOT VALID;
  ELSIF false THEN BEGIN
   ALTER TABLE "Mý"."Chïld" ADD CONSTRAINT "UK雪" UNIQUE ("Id");
  END;
  ELSE
   IF false THEN END IF;
   ALTER TABLE "Mý"."Chïld" ADD CONSTRAINT "CK雪" CHECK ("Id" > 0);
   INSERT INTO "Mý"."Chïld" ("Id") VALUES (1);
  END IF;
 END IF;
END $outer$;
CREATE INDEX after_constraint_idx ON "Mý"."Parent"("Id");
