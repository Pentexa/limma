import type { ID } from "@/shared/types/common";

/** User entity */
export interface User {
  id: ID;
  name: string;
  email: string;
  avatar: string | null;
  role: "admin" | "analyst" | "viewer";
}
