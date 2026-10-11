import { defineConfig, mergeConfig } from "vitest/config";
import { test, vi } from "vitest";

// A recognized config consumer cannot cancel an additional unknown callback consumer.
const escaped = { test: { testTimeout: 1 } };
const escapedCallback = () => { return escaped; };
unknownConsumer(escapedCallback);
defineConfig(escapedCallback);

const exported = { test: { testTimeout: 1 } };
export const exportedCallback = () => { return exported; };
defineConfig(exportedCallback);

const registered = { test: { testTimeout: 1 } };
const registeredCallback = () => { return registered; };
test("unknown return consumer", registeredCallback);
defineConfig(registeredCallback);

const operand = { test: { testTimeout: 1 } };
const operandCallback = () => { return operand; };
mergeConfig(operandCallback, {});
defineConfig(operand);

const effectful = { test: { testTimeout: 1 } };
const effectfulCallback = () => { unknownEffect(); return effectful; };
defineConfig(effectful);

const asynchronous = { test: { testTimeout: 1 } };
const asynchronousCallback = async () => { return asynchronous; };
defineConfig(asynchronous);

const generated = { test: { testTimeout: 1 } };
const generatorCallback = function* () { return generated; };
defineConfig(generated);

const parameterEffect = { test: { testTimeout: 1 } };
const parameterCallback = (env = unknownEffect()) => { return parameterEffect; };
defineConfig(parameterEffect);

const mutated = { test: { testTimeout: 1 } };
const mutatedCallback = () => { return mutated; };
mutated.test.testTimeout = 5001;
defineConfig(mutatedCallback);

const container = { test: { testTimeout: 1 } };
const containerCallback = () => { return container; };
const holder = { callback: containerCallback };
defineConfig(holder);
defineConfig(container);

const invoked = { test: { testTimeout: 1 } };
const invokedCallback = () => { return invoked; };
invokedCallback();
defineConfig(invokedCallback);

const secondArgument = { test: { testTimeout: 1 } };
const secondCallback = () => { return secondArgument; };
defineConfig({}, secondCallback);
defineConfig(secondArgument);

const destructured = { test: { testTimeout: 1 } };
const destructuredCallback = ({ mode }) => { return destructured; };
defineConfig(destructured);

const runtimeArgument = { test: { testTimeout: 1 } };
vi.setConfig({}, runtimeArgument);
defineConfig(runtimeArgument);
