import { Skeleton } from "@/shared/ui/skeleton";

export default function Loading() {
  return (
    <div className="h-screen w-full flex items-center justify-center bg-background">
      <div className="flex flex-col items-center gap-4">
        <div className="h-8 w-8 rounded-full border-2 border-primary border-t-transparent animate-spin" />
        <Skeleton className="h-4 w-32" />
      </div>
    </div>
  );
}
