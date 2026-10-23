import { test, expect } from '@playwright/test';

test('one record', async ({ page }) => {
  await page.goto('');

  const add_button = page.getByRole('button', { name: 'Add' });
  await add_button.click();
  const row = page.getByRole('row').filter({ has: page.getByRole('cell') });
  await row.click();

  await page.getByLabel('Title').fill('The Recursive Book of Recursion');
  await page.getByLabel('Author').fill('Al Sweigart');
  await row.click();  // blur input
  expect.soft(row).toContainText('The Recursive Book of Recursion');
  expect.soft(row).toContainText('Al Sweigart');
})
