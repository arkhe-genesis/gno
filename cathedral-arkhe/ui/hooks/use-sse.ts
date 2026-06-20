import { useEffect } from 'react';

export function useSSE(url: string, onMessage: (data: any) => void) {
  useEffect(() => {
    const eventSource = new EventSource(url);
    eventSource.onmessage = (event) => {
      try { onMessage(JSON.parse(event.data)); } catch (e) {}
    };
    return () => eventSource.close();
  }, [url, onMessage]);
}
