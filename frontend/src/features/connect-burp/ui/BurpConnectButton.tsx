"use client";

import { Button } from "@/shared/ui/button";
import { Link2 } from "lucide-react";

interface BurpConnectButtonProps { onClick?: () => void; }

export function BurpConnectButton({ onClick }: BurpConnectButtonProps) {
  return (
    <Button variant="outline" size="sm" className="gap-2 text-xs" onClick={onClick}>
      <Link2 className="h-3.5 w-3.5" />
      Burp Suite
    </Button>
  );
}
