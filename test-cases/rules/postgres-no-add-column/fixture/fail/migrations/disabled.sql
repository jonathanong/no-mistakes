-- no-mistakes-disable-next-line postgres-no-add-column: deployed schema exception
ALTER TABLE posts ADD COLUMN status text;
