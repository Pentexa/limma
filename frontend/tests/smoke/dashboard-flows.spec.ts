import { expect, test, type Page } from "@playwright/test";

const RULE_STATUS = {
  total_rules: 1,
  disabled_packs: [],
  disabled_rules: [],
  active_rules: [
    {
      id: "custom-header-check",
      name: "Custom Header Check",
      category: "misconfiguration",
      pack: "custom",
      source: "user-custom",
      version: "1.0.0",
      default_severity: "medium",
      default_confidence: "tentative",
      is_active: true,
    },
  ],
  feedback_stats: {},
};

const EMPTY_FEEDBACK_STATS = {
  total_feedback_entries: 0,
  rule_stats: {},
  recent_feedback: [],
};

async function mockBackend(page: Page) {
  await page.route("http://localhost:8900/**", async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    let body: unknown = [];

    if (pathname === "/api/rule-engine-status") body = RULE_STATUS;
    if (pathname === "/api/feedback-stats") body = EMPTY_FEEDBACK_STATS;

    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(body),
    });
  });
}

test.beforeEach(async ({ page }) => {
  await mockBackend(page);
});

test("scanner route renders its module navigation", async ({ page }) => {
  await page.goto("/scanner");

  await expect(page.getByRole("heading", { name: "Scanner" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Analyze" })).toBeVisible();
  await expect(page.getByRole("button", { name: "API Discovery" })).toBeVisible();
  await expect(page.getByText("No active scan target")).toBeVisible();
});

test("rules route opens the accessible custom-rule dialog", async ({ page }) => {
  await page.goto("/rules");

  await expect(page.getByRole("heading", { name: "Rule Engine" })).toBeVisible();
  await page.getByRole("button", { name: "New Rule" }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Create Custom Rule" })).toBeVisible();
});

test("rules route confirms custom-rule deletion in an accessible dialog", async ({ page }) => {
  await page.goto("/rules");

  await page.getByText("Custom Header Check").click();
  await page.getByRole("button", { name: "Delete" }).click();

  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Delete custom rule?" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Delete Rule" })).toBeVisible();
});

test("settings route switches between profile tabs", async ({ page }) => {
  await page.goto("/settings");

  await expect(page.getByRole("heading", { name: "System Settings" })).toBeVisible();
  await expect(page.getByText("No settings profiles found.")).toBeVisible();

  await page.getByRole("button", { name: /Scan Profiles/ }).click();
  await expect(page.getByText("No scan profiles found.")).toBeVisible();
});
