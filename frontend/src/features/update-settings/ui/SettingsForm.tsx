"use client";

import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Settings } from "lucide-react";

export function SettingsForm() {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Settings className="h-4 w-4" />
          Application Settings
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <label className="text-sm text-muted-foreground">API Base URL</label>
          <Input defaultValue="http://localhost:8900" className="h-8 text-sm" />
        </div>
        <div className="space-y-2">
          <label className="text-sm text-muted-foreground">Max Concurrent Scans</label>
          <Input type="number" defaultValue="3" className="h-8 text-sm w-24" />
        </div>
        <Button size="sm">Save Changes</Button>
      </CardContent>
    </Card>
  );
}
