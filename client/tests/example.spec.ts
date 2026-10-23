import { test, expect } from '@playwright/test';

test('has title', async ({ page }) => {
  await page.goto('');

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Memex/);
});

test('get started link', async ({ page }) => {
  await page.goto('');

  // await expect.soft(page.getByRole('heading', { name: 'Failure' })).toBeVisible();
  await expect.soft(page.getByRole('heading', { name: 'Search' })).toBeVisible();
  await expect.soft(page.getByRole('heading', { name: 'Details' })).toBeVisible();
  await expect.soft(page.getByRole('button', { name: 'Add' })).toBeVisible();
  // await expect.soft(page.getByRole('heading', { name: 'Bar' })).toBeVisible();
});
