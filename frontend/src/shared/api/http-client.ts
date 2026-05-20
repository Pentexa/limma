import { API_BASE_URL } from "@/shared/config/constants";
import type { ApiError } from "@/shared/types/common";

/** HTTP methods */
type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";

/** Request options */
interface RequestOptions {
  headers?: Record<string, string>;
  signal?: AbortSignal;
  params?: Record<string, string | number | boolean | undefined>;
}

/** Custom error class for API errors */
export class HttpError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = "HttpError";
  }
}

/** Build URL with query parameters */
function buildUrl(
  path: string,
  params?: Record<string, string | number | boolean | undefined>
): string {
  const url = new URL(path, API_BASE_URL);
  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        url.searchParams.set(key, String(value));
      }
    });
  }
  return url.toString();
}

/** Core fetch wrapper with typed responses and error handling */
async function request<T>(
  method: HttpMethod,
  path: string,
  body?: unknown,
  options?: RequestOptions
): Promise<T> {
  const url = buildUrl(path, options?.params);

  const response = await fetch(url, {
    method,
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
    body: body ? JSON.stringify(body) : undefined,
    signal: options?.signal,
    cache: "no-store",
  });

  if (!response.ok) {
    let errorData: Partial<ApiError> & Record<string, unknown>;
    try {
      errorData = await response.json();
    } catch {
      errorData = {
        message: response.statusText || "Unknown Error",
        code: "UNKNOWN_ERROR",
        status: response.status,
      };
    }

    // Handle APIs that use different error shapes (e.g. FastAPI's "detail")
    const message = errorData.message || (typeof errorData.detail === "string" ? errorData.detail : response.statusText) || "Unknown HTTP Error";
    const code = errorData.code || "API_ERROR";

    throw new HttpError(
      response.status,
      code,
      message,
      errorData.details
    );
  }

  // Handle 204 No Content — safe when T = void (default for patch/delete)
  if (response.status === 204) {
    return undefined as T;
  }

  const json = await response.json();

  // Unwrap backend envelope: { value: [...], Count: N } → [...]
  const unwrapped = unwrapEnvelope(json);

  return unwrapped as T;
}

/**
 * Unwrap common backend response envelopes.
 * Handles: { value: [...] }, { data: [...] }, { results: [...] },
 *          { items: [...] }, { findings: [...] }, { issues: [...] }
 * If no envelope detected, returns the original response unchanged.
 */
function unwrapEnvelope(json: unknown): unknown {
  if (json === null || json === undefined || typeof json !== "object" || Array.isArray(json)) {
    return json;
  }

  const obj = json as Record<string, unknown>;
  const envelopeKeys = ["value", "data", "results", "items", "findings", "issues"];

  for (const key of envelopeKeys) {
    if (Array.isArray(obj[key])) {
      return obj[key];
    }
  }

  // Also handle single-object envelopes like { data: { ... } }
  if ("data" in obj && typeof obj.data === "object" && obj.data !== null && !Array.isArray(obj.data)) {
    return obj.data;
  }

  return json;
}

/** Typed HTTP client */
export const httpClient = {
  get<T>(path: string, options?: RequestOptions): Promise<T> {
    return request<T>("GET", path, undefined, options);
  },

  post<T>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return request<T>("POST", path, body, options);
  },

  put<T>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return request<T>("PUT", path, body, options);
  },

  patch<T = void>(path: string, body?: unknown, options?: RequestOptions): Promise<T> {
    return request<T>("PATCH", path, body, options);
  },

  delete<T = void>(path: string, options?: RequestOptions): Promise<T> {
    return request<T>("DELETE", path, undefined, options);
  },
};
