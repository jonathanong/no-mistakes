import { test } from '@playwright/test';
import { navigateTo } from '../helpers/navigation';

test('view user', async ({ page }) => {
  await navigateTo(page, '/users/42');
});
