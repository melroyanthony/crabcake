"use client";

import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

import { LogoutButton } from "@/components/logout-button";
import { readMeOptions } from "@/lib/queries";
import { cn } from "@/lib/utils";

const links = [
  { href: "/dashboard", label: "Dashboard" },
  { href: "/items", label: "Items" },
  { href: "/settings", label: "Settings" },
] as const;

export function AppShell({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  const { data: user } = useQuery(readMeOptions());

  return (
    <div className="flex min-h-full flex-1 flex-col">
      <header className="border-b">
        <div className="mx-auto flex h-14 w-full max-w-5xl items-center gap-6 px-6">
          <Link href="/dashboard" className="font-mono text-sm font-medium">
            Crabcake
          </Link>
          <nav className="flex flex-1 items-center gap-1 text-sm">
            {links.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={cn(
                  "rounded-md px-2.5 py-1.5 text-muted-foreground transition-colors hover:text-foreground",
                  pathname.startsWith(link.href) &&
                    "bg-muted font-medium text-foreground",
                )}
              >
                {link.label}
              </Link>
            ))}
            {user?.is_superuser ? (
              <Link
                href="/admin"
                className={cn(
                  "rounded-md px-2.5 py-1.5 text-muted-foreground transition-colors hover:text-foreground",
                  pathname.startsWith("/admin") &&
                    "bg-muted font-medium text-foreground",
                )}
              >
                Admin
              </Link>
            ) : null}
          </nav>
          <LogoutButton variant="ghost" />
        </div>
      </header>
      <div className="mx-auto w-full max-w-5xl flex-1 px-6 py-10">
        {children}
      </div>
    </div>
  );
}
