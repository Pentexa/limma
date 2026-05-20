"use client";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/shared/ui/select";
import type { ScanProfile } from "../model/types";

interface ProfileSelectorProps {
  profiles: ScanProfile[];
  value?: string;
  onChange: (profileId: string) => void;
}

export function ProfileSelector({ profiles, value, onChange }: ProfileSelectorProps) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-[200px]">
        <SelectValue placeholder="Select profile" />
      </SelectTrigger>
      <SelectContent>
        {profiles.map((p) => (
          <SelectItem key={p.id} value={p.id}>{p.name}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
