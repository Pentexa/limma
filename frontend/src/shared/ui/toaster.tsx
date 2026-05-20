"use client";

import { Toaster as Sonner } from "sonner";
import { cn } from "@/shared/lib/utils";

export function Toaster() {
  return (
    <Sonner
      position="bottom-right"
      toastOptions={{
        className: cn(
          "bg-card border border-border text-foreground font-sans",
          "shadow-lg rounded-md"
        ),
        classNames: {
          toast: "group toast",
          description: "text-muted-foreground",
          actionButton: "bg-primary text-primary-foreground",
          cancelButton: "bg-muted text-muted-foreground",
          error: "border-risk/30 bg-risk/5 text-risk",
          success: "border-verified/30 bg-verified/5 text-verified",
          warning: "border-attention/30 bg-attention/5 text-attention",
          info: "border-primary/30 bg-primary/5 text-primary",
        },
      }}
    />
  );
}
