const state = {
  cleaned: [] as string[],
  dynamic: [] as string[],
  leaking: [] as string[],
};
const key = "dynamic";

afterEach(() => {
  state["cleaned"].length = 0;
});

test("leaks sibling state", () => {
  state.leaking.push("value");
});

test("leaks dynamic state", () => {
  state[key].push("value");
});
