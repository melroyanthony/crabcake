import { LoginForm } from "@/app/login/login-form";
import { AuthShell } from "@/components/auth-shell";

export const metadata = {
  title: "Sign in",
};

type Props = {
  searchParams: Promise<{ next?: string }>;
};

export default async function LoginPage({ searchParams }: Props) {
  const { next } = await searchParams;

  return (
    <AuthShell
      title="Sign in"
      description="Sessions live in httpOnly cookies. The browser never sees a token."
    >
      <LoginForm next={next ?? "/dashboard"} />
    </AuthShell>
  );
}
