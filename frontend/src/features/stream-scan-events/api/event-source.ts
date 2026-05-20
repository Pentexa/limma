import { connectToScanStream, parseSSEMessage } from "@/shared/api/sse-client";
import { useStreamStore } from "../model/stream-store";

/** Connect to the scan SSE stream and pipe events into the Zustand store */
export function connectScanStream(targetUrl?: string): () => void {
  const store = useStreamStore.getState();
  store.setConnectionStatus("connecting");

  return connectToScanStream({
    targetUrl,
    onOpen: () => {
      store.setConnectionStatus("connected");
    },
    onMessage: (event) => {
      const message = parseSSEMessage(event);
      store.addEvent(message);
    },
    onError: () => {
      store.setConnectionStatus("error");
    },
  });
}
