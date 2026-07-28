import Link from "next/link";

import { ResetForm } from "@/app/reset-password/reset-form";
import { AuthShell } from "@/components/auth-shell";

export const metadata = {
  title: "Reset password",
};

type Props = {
  searchParams: Promise<{ token?: string }>;
};

export default async function ResetPasswordPage({ searchParams }: Props) {
  const { token } = await searchParams;

  if (!token) {
    return (
      <AuthShell
        title="Reset password"
        description="This link is missing its token. Request a new one from the recover page."
      >
        <p className="text-center text-sm text-muted-foreground">
          <Link
            href="/recover-password"
            className="font-medium text-foreground underline-offset-4 hover:underline"
          >
            Recover password
          </Link>
        </p>
      </AuthShell>
    );
  }

  return (
    <AuthShell
      title="Choose a new password"
      description="This ends every other signed-in session for the account."
    >
      <ResetForm token={token} />
    </AuthShell>
  );
}
