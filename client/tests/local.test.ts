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
  await expect.soft(row).toContainText('The Recursive Book of Recursion');
  await expect.soft(row).toContainText('Al Sweigart');
})

test('two records', async ({ page }) => {
  await page.goto('');

  const add_button = page.getByRole('button', { name: 'Add', exact: true });
  await add_button.click();
  const first = page.getByRole('row').filter({ has: page.getByRole('cell') });
  await first.click();

  await page.getByLabel('Title').fill('The Recursive Book of Recursion');
  await page.getByLabel('Author').fill('Al Sweigart');
  await first.click();  // blur input
  await expect.soft(first).toContainText('The Recursive Book of Recursion');
  await expect.soft(first).toContainText('Al Sweigart');

  add_button.click();
  const second = page.getByRole('row').filter({ has: page.getByRole('cell'), hasNotText: 'Sweigart' });
  await second.click();
  await page.getByLabel('Title').fill('SICP');
  await page.getByLabel('Author').fill('Hal Abelson &al');
  await expect.soft(second).toContainText('SICP');
  await expect.soft(second).toContainText('Hal Abelson');
  // first row still intact
  await expect.soft(page.getByRole('row').filter({ has: page.getByRole('cell'), hasText: 'Sweigart' })).toContainText('The Recursive Book of Recursion');
})
})
