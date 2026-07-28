import { LoginForm } from "@/app/login/login-form";

export const metadata = {
  title: "Sign in",
};

type Props = {
  searchParams: Promise<{ next?: string }>;
};

export default async function LoginPage({ searchParams }: Props) {
  const { next } = await searchParams;

  return (
    <main className="mx-auto flex w-full max-w-sm flex-1 flex-col justify-center gap-8 px-6 py-16">
      <div className="space-y-2">
        <p className="font-mono text-sm text-muted-foreground">Crabcake</p>
        <h1 className="text-2xl font-semibold tracking-tight">Sign in</h1>
        <p className="text-sm text-muted-foreground">
          Sessions live in httpOnly cookies. The browser never sees a token.
        </p>
      </div>
      <LoginForm next={next ?? "/dashboard"} />
    </main>
  );
}
