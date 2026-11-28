import { test as base } from "@playwright/test";
import { relay } from "./wrapper-relay";

export const test = relay(base);
