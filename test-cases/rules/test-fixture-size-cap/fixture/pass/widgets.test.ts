const boundary = Array.from({ length: 1_000 }, () => ({ name: "widget" }));
const width = Array.from({ length: 1_024 }, () => ({ name: "widget" }));
const exact = new Array(1000);
const listed = new Array(1, 2, 3);
it.each([{ port: 5432 }])("record %j", () => {});
it.each([10, 100])("small %i", () => {});
