/**
 * Auth Helper — handles login/registration to bypass security during E2E tests.
 */
import { Page, expect } from '@playwright/test';

const DEFAULT_EMAIL = 'admin@gmail.com';
const DEFAULT_PASS = 'Admintest1';

/**
 * Ensures the test user is logged in.
 * Tries registration first, then login.
 */
export async function ensureLoggedIn(page: Page): Promise<void> {
  await page.goto('/auth/login');
  
  // Check if we are already logged in (redirected to /)
  const url = page.url();
  if (!url.includes('/auth/login')) {
    return;
  }

  // Try to login
  await page.getByLabel('Email Address').fill(DEFAULT_EMAIL);
  await page.getByLabel('Password').fill(DEFAULT_PASS);
  await page.click('button:has-text("Sign In")');

  // Wait for navigation: either redirect to dashboard or stay on login (error)
  try {
    await page.waitForURL('**/', { timeout: 10_000 });
    console.log('[Auth] Logged in successfully');
    return;
  } catch {
    // Still on login page — credentials may be wrong or user doesn't exist
  }

  const currentUrl = page.url();

  if (currentUrl.includes('/auth/login')) {
    // Login failed, try registration
    console.log('[Auth] Login failed, attempting auto-registration...');
    await page.click('text=Create Account');
    await page.waitForURL('**/auth/register');
    
    await page.getByLabel('Full Name').fill('Test User');
    await page.getByLabel('Email Address').fill(DEFAULT_EMAIL);
    await page.getByLabel('Password', { exact: true }).fill(DEFAULT_PASS);
    await page.getByLabel('Confirm Password').fill(DEFAULT_PASS);
    await page.click('button:has-text("Create Account")');
    
    // Should auto-login after registration
    await page.waitForURL('**/');
  } else {
    console.log('[Auth] Logged in successfully');
  }

  // Ensure sidebar or dashboard is visible
  await expect(page.locator('.sidebar')).toBeVisible();
  await expect(page.locator('h1')).toBeVisible();
  await page.waitForLoadState('networkidle');
}
