import { UsersTable } from "@/app/(app)/admin/users-table";

export const metadata = {
  title: "Admin",
};

export default function AdminPage() {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Users</h1>
        <p className="text-muted-foreground">
          Manage accounts, privileges, and active status.
        </p>
      </div>
      <UsersTable />
    </div>
  );
}
