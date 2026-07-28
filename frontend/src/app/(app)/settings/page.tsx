import { PasswordForm } from "@/app/(app)/settings/password-form";
import { ProfileForm } from "@/app/(app)/settings/profile-form";
import { SessionActions } from "@/app/(app)/settings/session-actions";
import { Separator } from "@/components/ui/separator";

export const metadata = {
  title: "Settings",
};

export default function SettingsPage() {
  return (
    <div className="space-y-10">
      <div className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Update your profile, password, and signed-in sessions.
        </p>
      </div>

      <section className="space-y-4">
        <h2 className="text-lg font-medium">Profile</h2>
        <ProfileForm />
      </section>

      <Separator />

      <section className="space-y-4">
        <h2 className="text-lg font-medium">Password</h2>
        <PasswordForm />
      </section>

      <Separator />

      <section className="space-y-4">
        <h2 className="text-lg font-medium">Sessions</h2>
        <SessionActions />
      </section>
    </div>
  );
}
