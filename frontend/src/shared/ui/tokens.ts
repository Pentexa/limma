/**
 * Design System Tokens for Limma UI
 * Centralizes magic numbers to ensure consistent spacing and typography.
 */

export const SPACING = {
  // Gaps
  gap: {
    sm: "gap-2",
    md: "gap-3",
    lg: "gap-4",
  },
  // Paddings
  panel: "p-3",
  panelHeader: "px-3 py-2",
  panelBody: "p-3",
  // Specific use-cases
  topbar: "px-4 py-2.5",
} as const;

export const TYPOGRAPHY = {
  // Global scales
  micro: "text-[9px]",
  tiny: "text-[10px]",
  small: "text-[11px]",
  base: "text-[12px]",
  large: "text-[14px]",
  
  // Specific semantic uses
  panelTitle: "text-[11px] font-bold uppercase tracking-widest",
  tableHeader: "text-[9px] font-semibold text-muted-foreground/70 uppercase tracking-widest",
  mono: "font-mono",
} as const;

/**
 * P2-006: Layout constants for commonly repeated magic numbers.
 * Use these instead of inline `w-[180px]`, `w-[320px]`, etc.
 */
export const LAYOUT = {
  /** Sidebar filter panel width */
  filterSidebar: "w-[180px]",
  /** Finding list panel in split views */
  findingList: "w-[320px]",
  /** Detail/inspector panel */
  detailPanel: "w-[340px]",
  /** Max width for truncated text in tables */
  truncateText: "max-w-[360px]",
  /** Default table page size */
  TABLE_PAGE_SIZE: 50,
  /** Event stream max display count */
  EVENT_STREAM_LIMIT: 50,
} as const;

