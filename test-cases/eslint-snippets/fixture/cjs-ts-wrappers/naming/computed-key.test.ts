import { vi } from "vitest";
import * as vt from "vitest";

vi["mock" as const]("./dep");
const { ["vi" as const]: api } = vt;
api.mock("./kept");
