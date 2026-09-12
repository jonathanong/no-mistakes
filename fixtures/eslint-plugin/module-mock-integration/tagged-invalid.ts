const foo = 1;
/* no-mistakes: integration=api */
export { foo as "not-valid" };
