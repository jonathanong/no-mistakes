/** @ts-ignore */
const a = 1;
/**
 * @ts-ignore */
const b = 2;
/*
 * @ts-expect-error */
const c = 3;
/**
 * @ts-ignore
 */
const d = 4;
/// @ts-expect-error: compatibility
const e = 5;
//// @ts-ignore should not act as a directive
const f = 6;
// / @ts-ignore separated slash is not TypeScript directive syntax
/* * @ts-ignore */
/* / @ts-expect-error */
// Whitespace after the opening block delimiter cannot restart its star prefix.
