"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { problemDetail } from "@/lib/errors";

export function SessionActions() {
  const router = useRouter();
  const [pending, setPending] = useState(false);

  async function logoutEverywhere() {
    setPending(true);
    try {
      const response = await fetch("/api/auth/logout-everywhere", {
        method: "POST",
      });
      if (!response.ok) {
        const problem = (await response.json().catch(() => null)) as {
          detail?: string;
        } | null;
        throw problem ?? new Error("logout failed");
      }
      toast.success("Signed out everywhere");
      router.replace("/login");
      router.refresh();
    } catch (err) {
      toast.error(problemDetail(err, "Could not sign out everywhere"));
      setPending(false);
    }
  }

  return (
    <div className="max-w-md space-y-3">
      <p className="text-sm text-muted-foreground">
        Revoke every refresh token for this account, including this browser.
      </p>
      <Button
        variant="destructive"
        disabled={pending}
        onClick={logoutEverywhere}
      >
        {pending ? "Signing out…" : "Sign out everywhere"}
      </Button>
    </div>
  );
}
