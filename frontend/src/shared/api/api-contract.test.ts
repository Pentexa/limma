import fs from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

type HttpMethod = "delete" | "get" | "patch" | "post" | "put";

interface Route {
  method: HttpMethod;
  path: string;
}

const HTTP_METHODS: HttpMethod[] = ["delete", "get", "patch", "post", "put"];
const FRONTEND_SOURCE_ROOT = path.resolve(process.cwd(), "src");
const BACKEND_ROUTER_FILE = path.resolve(process.cwd(), "../backend/src/main.rs");

function sourceFiles(root: string): string[] {
  return fs.readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) return sourceFiles(entryPath);
    return /\.(ts|tsx)$/.test(entry.name) && !entry.name.endsWith(".test.ts") ? [entryPath] : [];
  });
}

function normalizeFrontendPath(routePath: string): string {
  return routePath
    .replace(/\$\{[^}]+\}/g, ":param")
    .replace(/:[^/]+/g, ":param");
}

function normalizeBackendPath(routePath: string): string {
  return routePath.replace(/:[^/]+/g, ":param");
}

function frontendRoutes(): Route[] {
  const callPattern = /httpClient\.(get|post|put|patch|delete)(?:<[^;]+?>)?\(\s*[`"]([^`"]+)[`"]/g;

  return sourceFiles(FRONTEND_SOURCE_ROOT).flatMap((file) => {
    const source = fs.readFileSync(file, "utf8");
    return [...source.matchAll(callPattern)].map((match) => ({
      method: match[1] as HttpMethod,
      path: normalizeFrontendPath(match[2]),
    }));
  });
}

function backendRoutes(): Route[] {
  const source = fs.readFileSync(BACKEND_ROUTER_FILE, "utf8");
  const routerStart = source.indexOf("let app = Router::new()");
  const routerEnd = source.indexOf(".with_state", routerStart);
  const router = source.slice(routerStart, routerEnd);
  const routePattern = /\.route\(\s*"([^"]+)"\s*,([\s\S]*?)(?=\n\s*\.route\(|$)/g;

  return [...router.matchAll(routePattern)].flatMap((match) => {
    const [, routePath, registration] = match;
    return HTTP_METHODS.filter((method) => new RegExp(`(?:routing::)?${method}\\s*\\(`).test(registration))
      .map((method) => ({ method, path: normalizeBackendPath(routePath) }));
  });
}

describe("frontend API contract", () => {
  it("only calls HTTP routes exposed by the backend router", () => {
    const backend = new Set(backendRoutes().map(({ method, path: routePath }) => `${method} ${routePath}`));
    const missing = frontendRoutes()
      .map(({ method, path: routePath }) => `${method} ${routePath}`)
      .filter((route, index, routes) => !backend.has(route) && routes.indexOf(route) === index);

    expect(missing, `Frontend calls without a backend route:\n${missing.join("\n")}`).toEqual([]);
  });
});
