CREATE TABLE public.child (parent_id uuid REFERENCES parent);
CREATE INDEX ON public.child(parent_id);
