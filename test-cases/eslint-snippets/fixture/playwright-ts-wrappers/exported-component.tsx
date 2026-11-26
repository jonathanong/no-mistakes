import type React from "react";

declare const memo: <T>(component: T) => T;

const Base = () => <div />;
export const Direct = () => <div />;
export const Card = Base as React.FC;
export default Base satisfies React.FC;
export const MemoCard = (memo as typeof memo)(() => <div />);
export const InstCard = (memo<any>)(() => <div />);
export const AppliedCard = memo.apply(null, [() => <div />] as const);
export const SafeCard = (memo as typeof memo)(() => <div data-pw="safe" />);
