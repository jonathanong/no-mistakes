-- no-mistakes-disable-next-line postgres-require-fk-on-delete: historical table
CREATE TABLE children (parent_id uuid REFERENCES parents(id));
