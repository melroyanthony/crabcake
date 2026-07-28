"use client";

import { useQuery } from "@tanstack/react-query";

import { readMeOptions } from "@/client/@tanstack/react-query.gen";

export function UserGreeting() {
  const { data, error, isPending } = useQuery(readMeOptions());

  if (isPending) {
    return <p className="text-muted-foreground">Loading your account…</p>;
  }

  if (error || !data) {
    return (
      <p className="text-destructive">
        Could not load your account. Try signing in again.
      </p>
    );
  }

  return (
    <p className="text-muted-foreground">
      Signed in as{" "}
      <span className="font-medium text-foreground">
        {data.full_name ?? data.email}
      </span>
      {data.is_superuser ? " · superuser" : null}
    </p>
  );
}
