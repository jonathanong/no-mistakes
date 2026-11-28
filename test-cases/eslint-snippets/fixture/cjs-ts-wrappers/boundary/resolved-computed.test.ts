const vt = require("vitest");
declare const key: string;
const { [key]: api } = vt;
api.mock("react");
