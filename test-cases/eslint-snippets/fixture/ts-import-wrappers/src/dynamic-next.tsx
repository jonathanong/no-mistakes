declare const nextModule: string;
const Head = require(nextModule);
export const A = () => <script src="/a.js" />;
void Head;
