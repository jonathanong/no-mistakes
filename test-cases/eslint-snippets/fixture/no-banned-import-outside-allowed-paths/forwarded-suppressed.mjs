// no-mistakes-disable-file no-banned-import-outside-allowed-paths -- protocol fixture
import { invalidateAll as clear } from "@acme/rate-limit";
clear.call(null);
