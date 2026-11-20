it.each([0, 1001, 1.5])("rejects invalid batch size %s", () => {});
it.each([-1, 3601, 1.5])("rejects invalid seconds %s", () => {});
it.each([0, -1, 30_001, NaN])("rejects invalid timeout %s", () => {});
it.each(["", "bad-date", 20261004])("rejects non-day value %s", () => {});
