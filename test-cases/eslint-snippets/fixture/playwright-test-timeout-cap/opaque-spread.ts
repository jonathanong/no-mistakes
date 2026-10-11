import { test } from '@playwright/test';
test.describe.configure({timeout:30001, ...opaque});
test.describe.configure({...opaque, timeout:30000});
