import { vi } from "vitest";
import { enqueue } from "@example/jobs";
vi["mocked"](enqueue);
