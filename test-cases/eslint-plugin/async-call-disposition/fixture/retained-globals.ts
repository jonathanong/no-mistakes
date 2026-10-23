import { enqueueEmail } from "@app/jobs";
vi.fn((id) => enqueueEmail(id));
jest.spyOn(service, "send").mockImplementationOnce((id) => enqueueEmail(id));
// A namespace's method is not the framework's vi/jest API.
vitest.fn((id) => enqueueEmail(id));
