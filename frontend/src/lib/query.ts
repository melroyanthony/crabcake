"use client";

import { QueryClient } from "@tanstack/react-query";

export function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // Auth cookies are short-lived; a minute of freshness is enough, and
        // keeps the dashboard from refetching on every focus during typing.
        staleTime: 60_000,
        refetchOnWindowFocus: false,
      },
    },
  });
}
