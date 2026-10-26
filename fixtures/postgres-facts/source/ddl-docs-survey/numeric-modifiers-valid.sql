CREATE CAST (integer[] AS bigint[]) WITHOUT FUNCTION;
CREATE CAST (decimal(10,2) AS text) WITH INOUT;
CREATE CAST (float(24) AS text) WITH INOUT;
CREATE CAST (double precision AS text) WITH INOUT;
CREATE CAST (signed AS text) WITH INOUT;
CREATE CAST (unsigned AS text) WITH INOUT;
CREATE CAST (zerofill AS text) WITH INOUT;
CREATE CAST (public.unsigned AS text) WITH INOUT;
CREATE CAST ("integer" AS text) WITH INOUT;
CREATE CAST ("double" AS text) WITH INOUT;
CREATE CAST (double AS text) WITH INOUT; -- Bare double is a valid custom type name.
