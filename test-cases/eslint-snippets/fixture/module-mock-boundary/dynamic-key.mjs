import * as ns from "vitest";

const key = "vi";
const { [key]: dynamic } = ns;
const { ["vi"]: mocked } = ns;

dynamic.mock("./internal", () => {});
mocked.mock("./internal", () => {});
