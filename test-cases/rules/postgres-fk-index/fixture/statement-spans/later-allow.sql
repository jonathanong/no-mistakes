ALTER TABLE "AllowFirst" ADD CONSTRAINT "allow_first" FOREIGN KEY ("personId") REFERENCES "Person"("id");
ALTER TABLE "AllowSecond" ADD CONSTRAINT "allow_second" FOREIGN KEY ("personId") REFERENCES "Person"("id"); -- fk-index-allow: later key deliberately reviewed
