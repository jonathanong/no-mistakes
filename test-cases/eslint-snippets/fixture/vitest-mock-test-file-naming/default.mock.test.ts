import vitest from "vitest";

declare const name: string;
vitest.mock("./default-is-not-vi");
vitest[name]("./dynamic");
