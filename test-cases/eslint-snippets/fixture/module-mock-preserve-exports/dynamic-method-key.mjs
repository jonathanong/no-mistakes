import { vi } from "vitest";

const mocking = vi;
const mock = "unmock";
const { [mock]: clear } = mocking;
clear("./internal", () => ({ charge() {} }));

const { mock: real } = mocking;
real("./internal", () => ({ charge() {} }));

const { ["mock"]: bracket } = mocking;
bracket("./kept", () => ({ charge() {} }));

const { [`mock`]: templated } = mocking;
templated("./named", () => ({ charge() {} }));
