import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export default function Home() {
  return (
    <main className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-8 px-6 py-16">
      <div className="space-y-3">
        <p className="font-mono text-sm text-muted-foreground">Crabcake</p>
        <h1 className="text-4xl font-semibold tracking-tight text-balance">
          Your API is ready. The dashboard comes next.
        </h1>
        <p className="max-w-xl text-lg text-muted-foreground text-pretty">
          This frontend talks to the Axum API. Auth, the generated client and
          the pages land in the milestones that follow; for now the stack builds
          and the UI primitives are in place.
        </p>
      </div>

      <div className="flex flex-wrap gap-3">
        <a className={cn(buttonVariants())} href="http://localhost:8000/docs">
          Open API docs
        </a>
        <a
          className={cn(buttonVariants({ variant: "outline" }))}
          href="http://localhost:8000/health"
        >
          Health check
        </a>
      </div>
    </main>
  );
}
