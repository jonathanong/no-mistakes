import { read } from '@example/db';
declare function unrelated(value: unknown): void;
// The configured executor escapes even though this module has no direct SQL call.
unrelated(read);
