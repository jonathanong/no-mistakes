declare const componentPackage: string;

const UI = require(componentPackage);
export const Panel = () => <UI.Action />;
export const Ready = () => <UI.Action data-pw="ok" />;
