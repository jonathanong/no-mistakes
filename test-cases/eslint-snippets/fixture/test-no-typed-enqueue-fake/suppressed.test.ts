// no-mistakes-disable-file test-no-typed-enqueue-fake -- Verifies the directive.
import { enqueueEmail } from "./queue";

vi.fn<typeof enqueueEmail>();
