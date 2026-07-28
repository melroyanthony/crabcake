"use client";

import { useQuery } from "@tanstack/react-query";
import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { itemsListOptions, readMeOptions } from "@/lib/queries";
import { cn } from "@/lib/utils";

export function DashboardView() {
  const me = useQuery(readMeOptions());
  const items = useQuery(itemsListOptions({ query: { skip: 0, limit: 5 } }));

  return (
    <div className="space-y-10">
      <div className="space-y-2">
        <h1 className="text-3xl font-semibold tracking-tight">Dashboard</h1>
        {me.isPending ? (
          <Skeleton className="h-5 w-64" />
        ) : me.error || !me.data ? (
          <p className="text-destructive">Could not load your account.</p>
        ) : (
          <p className="text-muted-foreground">
            Signed in as{" "}
            <span className="font-medium text-foreground">
              {me.data.full_name ?? me.data.email}
            </span>
            {me.data.is_superuser ? " · superuser" : null}
          </p>
        )}
      </div>

      <section className="space-y-4">
        <div className="flex items-center justify-between gap-4">
          <h2 className="text-lg font-medium">Recent items</h2>
          <Link
            className={cn(buttonVariants({ variant: "outline", size: "sm" }))}
            href="/items"
          >
            View all
          </Link>
        </div>
        {items.isPending ? (
          <div className="space-y-2">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
        ) : items.error ? (
          <p className="text-sm text-destructive">Could not load items.</p>
        ) : items.data.data.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No items yet.{" "}
            <Link href="/items" className="underline underline-offset-4">
              Create one
            </Link>
            .
          </p>
        ) : (
          <ul className="divide-y rounded-lg border">
            {items.data.data.map((item) => (
              <li key={item.id} className="px-4 py-3">
                <p className="font-medium">{item.title}</p>
                {item.description ? (
                  <p className="truncate text-sm text-muted-foreground">
                    {item.description}
                  </p>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
