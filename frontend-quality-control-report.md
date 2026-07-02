# Frontend Quality Control Report

Date: 2026-06-20
Scope: `frontend` Next.js application
Final status: Complete - all findings in this report are resolved and verified.

## Executive Summary

The initial frontend quality findings have been implemented and the quality gate is now clean. ESLint, strict TypeScript, unit tests, Playwright smoke tests, production build, and the full dependency audit all pass.

Highest priority items addressed:

1. `useGlobalFindings()` can automatically POST to `/master-report` from globally mounted UI such as `TopBar` and `Sidebar`.
2. Starting a scan opens the SSE stream before the active scan is confirmed, but failure paths do not stop the stream.
3. Custom rule creation generates YAML via raw string interpolation with unescaped user input.
4. Scan profile editing appears to save to the wrong endpoint and invalidates the wrong query.
5. `npm audit` reports 10 production dependency advisories, including high-severity `next`, `fast-uri`, and `hono` advisories.

## Checks Run

| Check | Result | Notes |
| --- | --- | --- |
| `npm.cmd run lint` | Passed | 0 errors, 0 warnings |
| `npx.cmd tsc --noEmit` | Passed | Strict TypeScript check completed |
| `npm.cmd run build` | Passed | Next.js 16.2.9 production build succeeds without Google Fonts network fetches |
| `npm.cmd audit --audit-level=moderate --cache .npm-cache` | Passed | 0 vulnerabilities, including development dependencies |
| `npm.cmd test` | Passed | 4 Vitest files, 7 unit/hook/component tests |
| `npm.cmd run test:smoke -- --workers=1` | Passed | 4 Playwright Chromium flows: scanner, rule creation/deletion, settings |

## Findings

Resolution note: the findings below describe the original audited state. All findings have been resolved by the implementation recorded in the final progress section.

### P0 - Global Master Report POST Can Fire From Layout Components

`useGlobalFindings()` always enables `useMasterReportFindings()` whenever it sees an active scan id and target URL. That hook calls `fetchMasterReportFindings()`, which sends `POST /master-report`.

This hook is used by globally mounted components:

- `frontend/src/widgets/topbar/TopBar.tsx:10`, `frontend/src/widgets/topbar/TopBar.tsx:51`
- `frontend/src/widgets/sidebar-navigation/Sidebar.tsx:10`, `frontend/src/widgets/sidebar-navigation/Sidebar.tsx:14`
- `frontend/src/app/(dashboard)/layout.tsx:14`

Relevant fetch path:

- `frontend/src/entities/finding/model/use-findings.ts:45`
- `frontend/src/entities/finding/model/use-findings.ts:76`
- `frontend/src/entities/finding/api/finding-api.ts:57`
- `frontend/src/entities/finding/api/finding-api.ts:60`

Impact: navigating inside the dashboard can trigger expensive analysis work without explicit user intent. It also makes loading states in `TopBar` depend on Master Report completion.

Recommendation: split active findings from master-report findings. Make Master Report opt-in or page-scoped, and keep layout components on lightweight counters/endpoints only.

### P0 - SSE Stream Lifecycle Leaks On Scan Start Failure

Both scan start UIs mark the scan as starting and open the SSE stream before the active scan request succeeds:

- `frontend/src/features/start-scan/ui/StartScanButton.tsx:27`
- `frontend/src/features/start-scan/ui/StartScanButton.tsx:30`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:64`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:65`

On error, they only reset local scan state:

- `frontend/src/features/start-scan/ui/StartScanButton.tsx:46`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:91`

`stopScanStream()` exists but is not called on those failure paths:

- `frontend/src/features/stream-scan-events/model/scan-stream-manager.ts:44`

Impact: if `/api/active-scan` fails, the app can leave an SSE connection/retry loop open for a scan that never started. UI can show idle while the connection is still alive.

Recommendation: call `stopScanStream()` in scan-start catch blocks, and consider starting SSE only after active scan confirmation unless passive scan streaming truly must begin earlier.

### P0 - Custom Rule YAML Is Built With Unescaped User Input

`CreateRuleModal` interpolates user-controlled values directly into YAML:

- `frontend/src/screens/rules/CreateRuleModal.tsx:29`
- `frontend/src/screens/rules/CreateRuleModal.tsx:32`
- `frontend/src/screens/rules/CreateRuleModal.tsx:38`
- `frontend/src/screens/rules/CreateRuleModal.tsx:59`

Examples:

- `conditionValue` is wrapped in quotes without escaping for text/header conditions.
- `status_code_in` inserts raw input inside `codes: [...]`.
- `name`, `category`, `severity`, and generated values are assembled by template string.

Impact: quotes, newlines, colon-heavy values, or YAML aliases can break generated rules or change meaning. Since rules are loaded into a security engine, malformed or injected YAML has high blast radius.

Recommendation: build a structured rule object and serialize with a YAML library, validate `status_code_in` as numeric arrays, and reject multiline/special characters where not supported.

### P1 - Scan Profile Editing Saves To The Settings Profile Endpoint

`SettingsScreen` fetches two different profile lists:

- settings profiles: `frontend/src/features/update-settings/model/use-settings.ts:17`
- scan profiles: `frontend/src/features/update-settings/model/use-settings.ts:25`

Both tabs use the same save handler:

- `frontend/src/screens/settings/SettingsScreen.tsx:337`
- `frontend/src/screens/settings/SettingsScreen.tsx:450`
- `frontend/src/screens/settings/SettingsScreen.tsx:500`

But `useUpdateProfile()` always writes to `/api/settings/profiles/{id}` and only invalidates `settingsKeys.profiles()`:

- `frontend/src/features/update-settings/model/use-settings.ts:33`
- `frontend/src/features/update-settings/model/use-settings.ts:35`

Impact: editing a scan profile fetched from `/api/profiles` likely sends the update to the wrong backend endpoint and does not refresh the scan profile query. Users may see "saved" behavior while the intended profile remains stale or unchanged.

Recommendation: create separate mutations for settings profiles and scan profiles, each with the matching endpoint and query invalidation.

### P1 - Lint Gate Is Broken

`npm.cmd run lint` fails with 2 errors and 25 warnings. The errors are explicit `any` usage:

- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:83`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:120`

Warnings include unused imports/variables:

- `frontend/src/screens/discovery/DiscoveryScreen.tsx:62`
- `frontend/src/screens/discovery/components/CertificateDiscoveryPanel.tsx:7`
- `frontend/src/screens/reports/ReportsScreen.tsx:4`
- `frontend/src/screens/reports/ReportsScreen.tsx:5`
- `frontend/src/screens/reports/ReportsScreen.tsx:7`
- `frontend/src/screens/reports/ReportsScreen.tsx:9`
- `frontend/src/widgets/sidebar-navigation/Sidebar.tsx:7`

There is also a Next.js image warning:

- `frontend/src/widgets/sidebar-navigation/Sidebar.tsx:27`

Impact: lint is not a usable CI gate right now. Real warnings are mixed with dead code, making regressions easier to miss.

Recommendation: replace the two `any` casts with `ActiveVulnType[]` and the scan mode union, remove unused imports/vars, and use `next/image` or remove the unused `Image` import.

### P1 - Production Build Depends On Network Fonts

Initial `next build` failed because `next/font/google` tried to fetch Google Fonts:

- `frontend/src/app/layout.tsx:3`
- `frontend/src/app/layout.tsx:9`
- `frontend/src/app/layout.tsx:15`

With network access, build succeeded.

Impact: CI, offline deployments, restricted enterprise networks, and reproducible builds can fail even when code is correct.

Recommendation: self-host `Inter` and `JetBrains Mono`, or use system font fallbacks without build-time network fetches.

### P1 - Dependency Audit Reports High-Severity Advisories

`npm audit --omit=dev --audit-level=moderate` reported 10 vulnerabilities:

- 3 high
- 6 moderate
- 1 low

High-severity packages/advisories include:

- `next` range includes several advisories; audit suggests `next@16.2.9` via `npm audit fix --force`
- `fast-uri <=3.1.1`
- `hono <=4.12.24`

Moderate items include `brace-expansion`, `ip-address`, `js-yaml`, `postcss`, and `qs`.

Impact: production dependency surface has known issues. The `next` item is especially important because this is an App Router app exposed as the frontend shell.

Recommendation: review the package tree, apply non-breaking `npm audit fix` first, then separately test the forced Next upgrade path because audit says it is outside the current stated dependency range.

### P2 - Passive Scan Errors Are Silently Swallowed

`startScan()` fires `startPassiveScan()` in the background and swallows any error:

- `frontend/src/features/start-scan/api/start-scan.ts:58`
- `frontend/src/features/start-scan/api/start-scan.ts:88`
- `frontend/src/features/start-scan/api/start-scan.ts:90`

Impact: users may see an active scan start while SSE/passive analysis never starts. Since the error is intentionally discarded, debugging missing stream events becomes difficult.

Recommendation: report passive scan failure to the stream store or toast/log it in a non-blocking way. If passive scan is optional, surface that degraded mode explicitly.

### P2 - History Empty-State Action Does Nothing

The empty history state renders a "Start a Scan" button:

- `frontend/src/screens/history/HistoryScreen.tsx:173`
- `frontend/src/screens/history/HistoryScreen.tsx:175`

There is no click handler. `router` is already available in the component:

- `frontend/src/screens/history/HistoryScreen.tsx:29`

Impact: first-time users get a primary recovery action that is inert.

Recommendation: wire it to `router.push("/scanner")` or the dashboard scanner entry point.

### P2 - Destructive History Deletes Have No Confirmation

Single delete and bulk delete execute immediately:

- `frontend/src/screens/history/HistoryScreen.tsx:41`
- `frontend/src/screens/history/HistoryScreen.tsx:64`
- `frontend/src/screens/history/HistoryScreen.tsx:296`
- `frontend/src/screens/history/HistoryScreen.tsx:488`

Impact: accidental clicks can delete active/history scan records without confirmation. This is especially risky because the UI intentionally offers bulk selection.

Recommendation: add a confirmation dialog, require explicit confirmation for bulk deletes, and consider optimistic undo only if backend supports recovery.

### P2 - Rule Mutations Lack User-Facing Error Feedback

`RuleEngineScreen` closes the create modal on success, but no error callback or toast is wired for create/delete failures:

- `frontend/src/screens/rules/RuleEngineScreen.tsx:121`
- `frontend/src/screens/rules/RuleEngineScreen.tsx:131`
- `frontend/src/features/manage-rules/model/use-rules.ts:45`
- `frontend/src/features/manage-rules/model/use-rules.ts:57`

Impact: failed rule creation/deletion can leave the user with no actionable feedback except the modal staying open or a disabled button resetting.

Recommendation: add mutation `onError` handling with `toast.error`, and show validation/API errors inside the modal.

### P2 - Manual Modal In Rules Page Is Not Accessible Enough

`CreateRuleModal` is implemented as a manual fixed overlay:

- `frontend/src/screens/rules/CreateRuleModal.tsx:63`
- `frontend/src/screens/rules/CreateRuleModal.tsx:69`

Unlike the shared Radix dialog, it does not provide focus trapping, Escape handling, aria dialog semantics, or focus return.

Impact: keyboard and screen-reader users can get a degraded modal experience, and focus may escape behind the overlay.

Recommendation: migrate it to the shared `Dialog` component used by `ModernScanConfigPanel` and `ExecutionLevelDialog`.

### P2 - Native Alerts And Confirms Create Inconsistent UX

Blocking browser dialogs are used in several user-facing flows:

- `frontend/src/features/start-scan/ui/StartScanButton.tsx:49`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:58`
- `frontend/src/widgets/scan-config/ModernScanConfigPanel.tsx:93`
- `frontend/src/screens/rules/RuleEngineScreen.tsx:131`
- `frontend/src/screens/poc-lab/PocLabScreen.tsx:88`

Impact: the rest of the app uses `sonner` toasts and custom dialogs. Native dialogs interrupt flow, are hard to style/localize, and behave inconsistently in embedded/browser contexts.

Recommendation: replace with shared confirmation dialogs and toast/error panels.

### P3 - `Button` Component Does Not Default To `type="button"`

The shared Button renders a native `button` but does not set a default type:

- `frontend/src/shared/ui/button.tsx:44`
- `frontend/src/shared/ui/button.tsx:54`
- `frontend/src/shared/ui/button.tsx:57`

Impact: any future use inside a form will default to `submit`, which can trigger accidental form submission. Some current components use raw buttons with explicit `type`, but this component is a common primitive.

Recommendation: default `type="button"` when `asChild` is false, and require explicit `type="submit"` for submit actions.

### P3 - Target Submit Icon Has No Wired Action In TopBar

`TargetInput` supports Enter/arrow submit:

- `frontend/src/widgets/topbar/TargetInput.tsx:21`
- `frontend/src/widgets/topbar/TargetInput.tsx:28`

But `TopBar` does not pass `onTargetSubmit`:

- `frontend/src/widgets/topbar/TopBar.tsx:113`

Impact: the arrow icon communicates an action but does nothing. Users must discover that scan start lives in the adjacent configure/start control.

Recommendation: either wire the submit action or remove the arrow affordance in this context.

### P3 - Reports Screen Contains Dead Code And A Read-Only Report Flow

`ReportsScreen` imports unused report generation/save dependencies:

- `frontend/src/screens/reports/ReportsScreen.tsx:4`
- `frontend/src/screens/reports/ReportsScreen.tsx:5`
- `frontend/src/screens/reports/ReportsScreen.tsx:7`
- `frontend/src/screens/reports/ReportsScreen.tsx:9`
- `frontend/src/screens/reports/ReportsScreen.tsx:11`
- `frontend/src/screens/reports/ReportsScreen.tsx:12`

`saveLocalReport()` exists but is only imported and not used:

- `frontend/src/entities/report/api/report-api.ts:16`

Impact: the page appears to have partially removed report-generation logic. This increases maintenance noise and makes the Reports feature look more complete in code than it is in UI.

Recommendation: remove dead imports, or restore explicit report-generation actions if that feature is intended.

## Positive Notes

- TypeScript strict check passes.
- Production build succeeds without build-time font network access.
- API calls are centralized through `httpClient`, which gives a good base for consistent error handling.
- React Query is used consistently for caching and polling.
- Most data-heavy tables include overflow handling, which helps with the dense security dashboard layout.

## Recommended Fix Order

- [x] Stop automatic Master Report calls from layout/global components.
- [x] Fix SSE cleanup on scan start failures.
- [x] Replace custom rule YAML string generation with structured serialization and validation.
- [x] Fix scan profile update endpoint/query invalidation.
- [x] Clear lint errors and warnings so lint can be a CI gate.
- [x] Address dependency audit advisories and retest build.
- [x] Self-host fonts or remove build-time Google Fonts fetch.
- [x] Add a minimal test setup: unit tests for mappers/hooks and smoke tests for scanner/rules/settings flows.

## Implementation Progress

Updated: 2026-06-20

- Fixed: global `useGlobalFindings()` no longer triggers `POST /master-report`.
- Fixed: scan start failure paths now call `stopScanStream()` and surface toast errors.
- Fixed: custom rule YAML is generated with the maintained `yaml` library from a structured object with validation for headers, multiline values, and status codes.
- Fixed: settings profile and scan profile updates now use separate mutations/endpoints and invalidations.
- Fixed: lint issues from explicit `any`, unused imports, and the sidebar image warning.
- Fixed: production build no longer depends on `next/font/google`; system font fallbacks are used.
- Fixed: `Button` defaults to `type="button"` when not rendered as child.
- Fixed: history empty-state action navigates to `/scanner`, and history deletes now ask for confirmation.
- Fixed: rule create/delete mutations now show success/error toasts.
- Fixed: native alerts and confirmations were replaced with Sonner feedback and a shared accessible confirmation dialog.
- Fixed: passive scan startup failures now surface a non-blocking warning while the active scan remains independent.
- Fixed: reports dead imports were removed.
- Fixed: Next and `eslint-config-next` were updated to `16.2.9`; PostCSS is overridden to `8.5.12` until Next updates its pinned dependency.
- Added: Vitest/Testing Library setup with mapper, `useGlobalFindings()`, and YAML serialization coverage.
- Added: Playwright smoke coverage for scanner navigation, accessible rule creation/deletion, and settings profile tabs.
- Verified: full audit reports 0 vulnerabilities; lint, typecheck, unit tests, smoke tests, and production build pass.
