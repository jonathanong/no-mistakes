const called = Array(10_000);
const exact = Array(1000);
const listed = Array(1, 2, 3);
it.each([10_000, "widget", 10_001])("mixed %s", () => {});
it.each([10_000, [10_001]])("nested %s", () => {});
it.each([10_000, , 10_001])("hole %s", () => {});
const rest = [1];
it.each([10_000, ...rest])("spread %s", () => {});
