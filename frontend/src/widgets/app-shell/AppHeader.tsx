"use client";

import { APP_NAME } from "@/shared/config/constants";
import { WorkspaceSwitcher } from "@/entities/workspace/ui/WorkspaceSwitcher";
import { UserMenu } from "@/entities/user/ui/UserMenu";
import { BurpConnectButton } from "@/features/connect-burp/ui/BurpConnectButton";

export function AppHeader() {
  return (
    <div className="flex items-center justify-between px-4 py-2">
      <div className="flex items-center gap-3">
        <h1 className="text-sm font-bold tracking-wider text-primary">{APP_NAME}</h1>
        <WorkspaceSwitcher />
      </div>
      <div className="flex items-center gap-2">
        <BurpConnectButton />
        <UserMenu />
      </div>
    </div>
  );
}
