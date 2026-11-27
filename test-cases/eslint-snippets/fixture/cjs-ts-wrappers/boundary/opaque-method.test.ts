import { vi } from "vitest";

const key = "ns";
vi[key].unmock("react");
vi[key].doUnmock("react");
vi[key].mock("./dep");
