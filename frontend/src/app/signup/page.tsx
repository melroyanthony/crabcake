import { SignupForm } from "@/app/signup/signup-form";
import { AuthShell } from "@/components/auth-shell";

export const metadata = {
  title: "Sign up",
};

export default function SignupPage() {
  return (
    <AuthShell
      title="Create an account"
      description="Open registration. Privileges are never set from this form."
    >
      <SignupForm />
    </AuthShell>
  );
}
