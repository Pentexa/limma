"use client";

import { Button } from "@/shared/ui/button";
import { User as UserIcon } from "lucide-react";

export function UserMenu() {
  return (
    <Button variant="ghost" size="icon" className="h-8 w-8 rounded-full">
      <UserIcon className="h-4 w-4 text-muted-foreground" />
    </Button>
  );
}
