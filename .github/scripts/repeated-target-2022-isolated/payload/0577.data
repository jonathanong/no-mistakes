CREATE TABLE public."café" (
  id integer DEFAULT check(1) PRIMARY KEY,
  parent_id integer REFERENCES public.parent(id),
  action_id integer REFERENCES public.parent(id) ON DELETE SET DEFAULT DEFAULT 1 NOT NULL,
  repeated integer UNIQUE UNIQUE,
  commented integer PRIMARY /* split keyword */ KEY,
  arrayed integer DEFAULT ARRAY[1, 2] UNIQUE,
  named_option integer UNIQUE CONSTRAINT named_not_null NOT NULL,
  CONSTRAINT "ck_name" CHECK (length('λ') > 0),
  UNIQUE (parent_id)
);
ALTER TABLE public."café"
  ADD CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES public.parent(id) NOT VALID,
  ADD CHECK (id > 0);
ALTER TABLE public."café" ENABLE ROW LEVEL SECURITY,
  ADD CONSTRAINT after_other_operation CHECK (parent_id > 0);
ALTER TABLE public."café" ALTER COLUMN parent_id SET DEFAULT ARRAY[1, 2],
  ADD CONSTRAINT after_array_default CHECK (parent_id > 0);
ALTER TABLE public."café" ADD COLUMN added integer CHECK (added > 0);

CREATE TABLE span_named_options (
  named_pk integer CONSTRAINT named_pk PRIMARY KEY,
  named_unique integer CONSTRAINT named_unique UNIQUE,
  named_fk integer CONSTRAINT named_fk REFERENCES public.parent(id),
  named_check integer CONSTRAINT named_check CHECK (named_check > 0),
  named_not_null integer UNIQUE CONSTRAINT named_not_null NOT NULL,
  null_then_unique integer NULL UNIQUE,
  generated_after_check integer CHECK (generated_after_check > 0) GENERATED ALWAYS AS (1) STORED,
  unsupported_after_unique text UNIQUE COLLATE "C"
);
