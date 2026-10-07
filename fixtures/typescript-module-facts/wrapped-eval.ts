// TypeScript wrappers disappear at runtime, preserving direct eval semantics.
declare const source: string;
eval!(source);
(eval as any)(source);
(eval satisfies Function)(source);
(<Function>eval)(source);
(eval)(source);
