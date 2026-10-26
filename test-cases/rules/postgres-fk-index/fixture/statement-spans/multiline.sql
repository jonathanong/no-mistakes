-- Earlier words must not claim a later declaration: personId references.
ALTER
TABLE "MultilineFirst"
ADD CONSTRAINT "multi_first" FOREIGN KEY ("personId") REFERENCES "Person"("id");
ALTER
TABLE "MultilineSecond"
ADD CONSTRAINT "multi_second" FOREIGN KEY ("personId") REFERENCES "Person"("id");
