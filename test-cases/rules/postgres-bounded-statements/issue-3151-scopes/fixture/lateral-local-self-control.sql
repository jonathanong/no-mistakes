SELECT public.number_kinds.id, local_row.label FROM public.number_kinds
CROSS JOIN LATERAL (
  SELECT number_kinds.label FROM number_kinds
  WHERE number_kinds.id = public.number_kinds.id
) local_row WHERE public.number_kinds.id = $1;
