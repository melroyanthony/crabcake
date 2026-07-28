import Link from "next/link";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export default function Home() {
  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-16">
      <div className="space-y-3">
        <p className="font-mono text-sm text-muted-foreground">Crabcake</p>
        <h1 className="text-4xl font-semibold tracking-tight text-balance">
          Your full-stack app, ready to customize.
        </h1>
        <p className="max-w-xl text-lg text-muted-foreground text-pretty">
          Sign in to manage items and settings. The API docs stay at the Axum
          origin for exploring the OpenAPI surface directly.
        </p>
      </div>

      <div className="flex flex-wrap gap-3">
        <Link className={cn(buttonVariants())} href="/login">
          Sign in
        </Link>
        <Link
          className={cn(buttonVariants({ variant: "outline" }))}
          href="/signup"
        >
          Create account
        </Link>
        <a
          className={cn(buttonVariants({ variant: "ghost" }))}
          href="http://localhost:8000/docs"
        >
          API docs
        </a>
      </div>
    </main>
  );
}
