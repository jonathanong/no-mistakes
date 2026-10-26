ALTER TABLE "First" ADD CONSTRAINT "first_person" FOREIGN KEY ("personId") REFERENCES "Person"("id");
-- no-mistakes-disable-next-line postgres-fk-index: later key deliberately reviewed
ALTER TABLE "Second" ADD CONSTRAINT "second_person" FOREIGN KEY ("personId") REFERENCES "Person"("id");
ALTER TABLE "Third" ADD CONSTRAINT "third_person" FOREIGN KEY ("personId") REFERENCES "Person"("id"); -- no-mistakes-disable-line postgres-fk-index: same-line exception
