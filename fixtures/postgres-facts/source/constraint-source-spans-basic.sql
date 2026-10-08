CREATE TABLE public.children (
  parent_id uuid REFERENCES public.parents(id),
  CONSTRAINT children_check CHECK (parent_id IS NOT NULL)
);
ALTER TABLE public.children
  ADD CONSTRAINT children_parent_fk
  FOREIGN KEY (parent_id) REFERENCES public.parents(id) NOT VALID;
