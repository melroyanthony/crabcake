import { RecoverForm } from "@/app/recover-password/recover-form";
import { AuthShell } from "@/components/auth-shell";

export const metadata = {
  title: "Recover password",
};

export default function RecoverPasswordPage() {
  return (
    <AuthShell
      title="Recover password"
      description="We always respond the same way, whether the email is known or not."
    >
      <RecoverForm />
    </AuthShell>
  );
}
