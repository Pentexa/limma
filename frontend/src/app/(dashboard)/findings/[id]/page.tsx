"use client";

import { use } from "react";
import { FindingDetailScreen } from "@/screens/finding-detail/FindingDetailScreen";

export default function FindingDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  return <FindingDetailScreen findingId={id} />;
}
