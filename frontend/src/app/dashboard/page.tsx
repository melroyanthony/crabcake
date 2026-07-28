import Link from "next/link";

import { LogoutButton } from "@/app/dashboard/logout-button";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const metadata = {
  title: "Dashboard",
};

/**
 * Placeholder behind the auth gate.
 *
 * Real dashboard content arrives with the pages milestone. This exists so the
 * BFF can be exercised end to end: login → cookie → protected page → logout.
 */
export default function DashboardPage() {
  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-8 px-6 py-16">
      <div className="space-y-2">
        <p className="font-mono text-sm text-muted-foreground">Crabcake</p>
        <h1 className="text-3xl font-semibold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          You are signed in. The session is carried by httpOnly cookies and
          refreshed through the same-origin API proxy.
        </p>
      </div>

      <div className="flex flex-wrap gap-3">
        <Link className={cn(buttonVariants({ variant: "outline" }))} href="/">
          Home
        </Link>
        <LogoutButton />
      </div>
    </main>
  );
}
