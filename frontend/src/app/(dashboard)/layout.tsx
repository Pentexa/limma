"use client";

import { AppShell } from "@/widgets/app-shell/AppShell";
import { Sidebar } from "@/widgets/sidebar-navigation/Sidebar";
import { TopBar } from "@/widgets/topbar/TopBar";
import { LiveStream } from "@/widgets/live-activity-stream/LiveStream";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <AppShell
      sidebar={<Sidebar />}
      topbar={<TopBar />}
      bottomPanel={<LiveStream />}
    >
      {children}
    </AppShell>
  );
}
