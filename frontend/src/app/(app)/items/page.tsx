import { ItemsTable } from "@/app/(app)/items/items-table";

export const metadata = {
  title: "Items",
};

export default function ItemsPage() {
  return (
    <div className="space-y-6">
      <div className="space-y-1">
        <h1 className="text-3xl font-semibold tracking-tight">Items</h1>
        <p className="text-muted-foreground">
          Create and manage the things that belong to your account.
        </p>
      </div>
      <ItemsTable />
    </div>
  );
}
