// A lookup table certifies identifiers only; calls and property reads still
// require resolution by the prepared analyzer, never textual fallback here.
const call = buildSql();
const property = object.sql;
const scalar = 42;
