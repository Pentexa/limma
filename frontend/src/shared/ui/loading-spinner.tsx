import { Loader2 } from "lucide-react";
import { cn } from "@/shared/lib/utils";

interface LoadingSpinnerProps {
  message?: string;
  className?: string;
  size?: "sm" | "md" | "lg";
}

export function LoadingSpinner({ message = "Loading...", className, size = "md" }: LoadingSpinnerProps) {
  const sizeClasses = {
    sm: "h-3 w-3",
    md: "h-4 w-4",
    lg: "h-6 w-6"
  };

  const textClasses = {
    sm: "text-[10px]",
    md: "text-[12px]",
    lg: "text-[14px]"
  };

  return (
    <div className={cn("flex items-center justify-center py-8", className)}>
      <div className="flex items-center gap-2 text-muted-foreground animate-pulse">
        <Loader2 className={cn("animate-spin", sizeClasses[size])} />
        <span className={textClasses[size]}>{message}</span>
      </div>
    </div>
  );
}
