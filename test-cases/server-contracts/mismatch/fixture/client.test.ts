test("fetches debug users", async () => {
  await fetch("/api/users?debug=1");
});
