/**
 * Shared UI barrel export.
 * P2-007: Standardized import paths for shared UI components.
 *
 * Usage: import { ErrorPanel, EmptyState, SkeletonPanel } from "@/shared/ui";
 */

// Layout & Feedback
export { ErrorPanel } from "./error-panel";
export { EmptyState } from "./empty-state";
export { SkeletonPanel } from "./skeleton-panel";
export { LoadingSpinner } from "./loading-spinner";
export { ErrorBoundary } from "./error-boundary";
export { ConfirmationDialog } from "./confirmation-dialog";

// Data Display
export { SeverityBadge, StatusBadge } from "./badges";

// Design Tokens
export { SPACING, TYPOGRAPHY } from "./tokens";
