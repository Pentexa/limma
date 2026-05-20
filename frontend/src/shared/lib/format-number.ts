const COMPACT_FORMATTER = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

const PERCENT_FORMATTER = new Intl.NumberFormat("en", {
  style: "percent",
  minimumFractionDigits: 0,
  maximumFractionDigits: 1,
});

/** Format a number in compact form (e.g. 1.2K, 3.4M) */
export function formatCompact(value: number): string {
  return COMPACT_FORMATTER.format(value);
}

/** Format a number as percentage (0-1 → 0%-100%) */
export function formatPercent(value: number): string {
  return PERCENT_FORMATTER.format(value);
}

/** Format bytes to human readable (e.g. 1.5 MB) */
export function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
}

/** Pad a number with leading zeros */
export function padNumber(value: number, length: number): string {
  return String(value).padStart(length, "0");
}
