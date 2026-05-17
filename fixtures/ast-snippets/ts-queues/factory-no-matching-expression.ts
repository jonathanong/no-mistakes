import { createQueue as cq } from "@factory/pkg";

const missing = undefined;
const notFactory = other("nope");
const wrapped = wrapper(other("nested"));
const member = factory.createQueue("member");
const casted = (other("casted") as unknown);
const nonNull = other("nonnull")!;
