CREATE TABLE public.constraint_kitchen (
  id uuid PRIMARY KEY,
  email text NOT NULL UNIQUE,
  nickname text NULL,
  parent_id uuid REFERENCES parents(id),
  score int CHECK (score > 0),
  created_at timestamptz DEFAULT now(),
  serial_id bigint GENERATED ALWAYS AS IDENTITY,
  computed text GENERATED ALWAYS AS (now()) STORED,
  nested_gen timestamptz GENERATED ALWAYS AS ((uuid_extract_timestamp(public.id))) STORED,
  skipped int
);

SELECT 1;
