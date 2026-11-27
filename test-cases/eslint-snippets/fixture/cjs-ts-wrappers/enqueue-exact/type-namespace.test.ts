import type * as V from "vitest";
import { enqueue } from "@example/jobs";

type Fake = V.MockedFunction<typeof enqueue>;
void Fake;
