declare const moduleName: string;
import UI from "./ui";
const Card = require(moduleName);
export const Panel = () => <UI.Card />;
export const Loose = () => <Card />;
