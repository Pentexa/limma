import type { Metadata } from "next";
import "./globals.css";
import { QueryProvider } from "@/shared/api/query-client";
import { TooltipProvider } from "@/shared/ui/tooltip";
import { Toaster } from "@/shared/ui/toaster";

export const metadata: Metadata = {
  title: "LIMMA — Security Auditing Platform",
  description:
    "Advanced security auditing and vulnerability detection platform with real-time scanning capabilities.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="antialiased font-sans">
        <QueryProvider>
          <TooltipProvider delayDuration={200}>
            {children}
            <Toaster />
          </TooltipProvider>
        </QueryProvider>
      </body>
    </html>
  );
}
