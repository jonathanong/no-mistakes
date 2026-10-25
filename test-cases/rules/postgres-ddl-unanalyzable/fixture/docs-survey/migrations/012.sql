CREATE FOREIGN TABLE precision_temporal (at timestamp(3) with time zone, opened time(6) without time zone, plain timestamp(0), local_time time(2)) SERVER server_name;
CREATE FOREIGN TABLE dimensional_arrays (tags text[][], ids int[3], grid integer[2][4], mixed text[][2][]) SERVER server_name;
SECURITY LABEL ON FUNCTION public.lookup(timestamp(3) with time zone, time(6) without time zone, text[][], int[3]) IS 'typed signature';
