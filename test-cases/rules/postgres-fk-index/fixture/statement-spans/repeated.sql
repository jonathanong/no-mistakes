ALTER TABLE "Attribute" ADD CONSTRAINT "Attribute_personId_fkey" FOREIGN KEY ("personId") REFERENCES "Person"("id");
ALTER TABLE "Session" ADD CONSTRAINT "Session_personId_fkey" FOREIGN KEY ("personId") REFERENCES "Person"("id");
