declare const componentName: string;

const { [componentName]: Mystery } = require("./ui");
export const Panel = () => <Mystery />;
export const Ready = () => <Mystery data-pw="ok" />;
