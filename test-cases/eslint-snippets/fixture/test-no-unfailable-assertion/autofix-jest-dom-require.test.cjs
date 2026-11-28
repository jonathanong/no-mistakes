require("@testing-library/jest-dom");
const { expect } = require("vitest");
const { screen } = require("@testing-library/react");

expect(screen.getByText("Save")).not.toBeNull();
