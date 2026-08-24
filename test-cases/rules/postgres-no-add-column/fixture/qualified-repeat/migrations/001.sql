ALTER TABLE public.posts ADD COLUMN status text NOT NULL DEFAULT 'draft';
ALTER TABLE audit.posts ADD COLUMN status text NOT NULL DEFAULT 'draft';
ALTER TABLE public.posts ADD COLUMN status text NOT NULL DEFAULT 'draft';
