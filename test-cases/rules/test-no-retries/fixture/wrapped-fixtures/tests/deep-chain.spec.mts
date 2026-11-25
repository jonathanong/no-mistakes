import { test as base } from "@playwright/test";

const f1 = () => base.extend({});
const f2 = () => f1();
const f3 = () => f2();
const f4 = () => f3();
const f5 = () => f4();
const f6 = () => f5();
const f7 = () => f6();
const f8 = () => f7();
const f9 = () => f8();

const t = f9();
t.describe.serial("ordered", () => {});
t.describe.configure({ retries: 2 });
