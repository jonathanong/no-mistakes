const { expect } = require("vitest");
const rtl = require("@testing-library/react");
const { screen } = require("@testing-library/react");

function checkSwitch(kind, user) {
  switch (kind) {
    case "a": {
      expect(rtl.screen.getByText("Ok")).toBeDefined();
      expect(screen.getByText("Ok")).toBeTruthy();
      expect(user).toBeDefined();
      expect(user?.name).toBe("Alice");
      break;
    }
    case "b":
      expect(user).toBeDefined();
      expect(user?.name).toBe("Bob");
      break;
  }
  expect(globalA).toBe(globalB);
  expect(globalA).toBe(globalA);
  expect(+1).toBe(1);
  expect(-user).toBe(0);
  expect().toBeDefined();
  expect(...user).toBeDefined();
  expect(1).toBe(...user);
  expect(user).custom.toBe(1);
  expect(user)[kind]();
  expect(globalUser).toBeDefined();
  expect(globalUser?.name).toBe("G");
}

module.exports = { checkSwitch };
