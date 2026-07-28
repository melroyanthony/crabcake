import Link from "next/link";

import { LogoutButton } from "@/app/dashboard/logout-button";
import { UserGreeting } from "@/app/dashboard/user-greeting";
import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export const metadata = {
  title: "Dashboard",
};

/**
 * Placeholder behind the auth gate.
 *
 * Real dashboard content arrives with the pages milestone. This exists so the
 * BFF and the generated client can be exercised end to end.
 */
export default function DashboardPage() {
  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col gap-8 px-6 py-16">
      <div className="space-y-2">
        <p className="font-mono text-sm text-muted-foreground">Crabcake</p>
        <h1 className="text-3xl font-semibold tracking-tight">Dashboard</h1>
        <UserGreeting />
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
