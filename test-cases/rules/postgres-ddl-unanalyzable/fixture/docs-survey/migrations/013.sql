SECURITY LABEL ON FUNCTION f(double text) IS 'named argument';
SECURITY LABEL ON FUNCTION f(IN first_arg integer, OUT second_arg text, INOUT third_arg numeric(12, 2), VARIADIC rest integer[]) IS 'argument modes';
SECURITY LABEL ON FUNCTION f(IN integer, OUT text, INOUT numeric(12, 2), VARIADIC integer[]) IS 'unnamed modes';
SECURITY LABEL ON FUNCTION f(value double) IS 'custom double type';
CREATE FOREIGN TABLE modifier_widths (v varchar(3), c char(2), n numeric(4, 2), d decimal(5), f float(24), i interval(3), custom public.custom_type(1, 2)) SERVER server_name;
