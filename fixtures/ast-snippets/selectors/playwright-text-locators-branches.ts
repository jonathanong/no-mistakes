test.describe("settings", () => {
  test("active role", async ({ page }) => {
    await page.getByRole(`button`, { name: `Save` }).click();
  });
});

test.skip("skipped text", async ({ page }) => {
  await page.getByText("Skip me").click();
});

if (process.env.E2E) {
  test("conditional label", async ({ page }) => {
    await page.getByLabel("Conditional label").fill("value");
  });
} else {
  test("conditional placeholder", async ({ page }) => {
    await page.getByPlaceholder("Conditional placeholder").fill("value");
  });
}

enabled && test("logical text", async ({ page }) => {
  await page.getByText("Logical text").click();
});

enabled
  ? test("ternary text", async ({ page }) => {
      await page.getByText("Ternary then").click();
    })
  : test("ternary other", async ({ page }) => {
      await page.getByText("Ternary else").click();
    });

test.skip(({ browserName }) => browserName === "webkit", "annotation");
test("annotation text", async ({ page }) => {
  await page.getByText("Annotation text").click();
  await page.getByText("Last exact text", { exact: false, exact: true }).click();
  await page.getByRole("button", { name: "Old name", name: "New name", includeHidden: true, includeHidden: false }).click();
  await page.getByRole("button", { 0: "zero", name: "Numeric property" }).click();
});

test("unsupported text locators", async ({ page }) => {
  await page.getByText(`Hello ${name}`).click();
  await page.getByRole("button", { ["name"]: "Computed" }).click();
  await page.getByRole("button", { name() { return "Method"; } }).click();
  await page.getByRole("checkbox", { [filterName]: true, name: "Dynamic filter" }).click();
  await page.getByRole("button", { title: "Wrong property" }).click();
  await page.getByRole("button", { name: /Regex/ }).click();
  await page.getByRole(name, { name: "Dynamic role" }).click();
});
