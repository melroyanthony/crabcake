"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { toast } from "sonner";

import { UserDialog } from "@/app/(app)/admin/user-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import type { UserPublic } from "@/lib/api";
import { problemDetail } from "@/lib/errors";
import {
  deleteUserMutation,
  readMeOptions,
  usersListOptions,
  usersListQueryKey,
} from "@/lib/queries";

const PAGE_SIZE = 20;

export function UsersTable() {
  const queryClient = useQueryClient();
  const me = useQuery(readMeOptions());
  const [skip, setSkip] = useState(0);
  const [editing, setEditing] = useState<UserPublic | null>(null);
  const [creating, setCreating] = useState(false);

  const list = useQuery(
    usersListOptions({ query: { skip, limit: PAGE_SIZE } }),
  );
  const remove = useMutation(deleteUserMutation());

  async function onDelete(user: UserPublic) {
    if (!window.confirm(`Delete ${user.email}?`)) {
      return;
    }
    try {
      await remove.mutateAsync({ path: { id: user.id } });
      toast.success("User deleted");
      await queryClient.invalidateQueries({ queryKey: usersListQueryKey() });
    } catch (err) {
      toast.error(problemDetail(err, "Could not delete user"));
    }
  }

  if (me.isPending) {
    return <Skeleton className="h-40 w-full" />;
  }

  if (!me.data?.is_superuser) {
    return (
      <p className="rounded-lg border border-dashed px-4 py-10 text-center text-sm text-muted-foreground">
        Superuser access is required for this page.
      </p>
    );
  }

  const count = list.data?.count ?? 0;
  const page = list.data?.data ?? [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <p className="text-sm text-muted-foreground">
          {list.isPending
            ? "Loading…"
            : `${count} user${count === 1 ? "" : "s"}`}
        </p>
        <Button onClick={() => setCreating(true)}>New user</Button>
      </div>

      {list.isPending ? (
        <div className="space-y-2">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      ) : list.error ? (
        <p className="text-sm text-destructive">Could not load users.</p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Email</TableHead>
              <TableHead className="hidden sm:table-cell">Name</TableHead>
              <TableHead>Flags</TableHead>
              <TableHead className="w-40 text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {page.map((user) => (
              <TableRow key={user.id}>
                <TableCell className="font-medium">{user.email}</TableCell>
                <TableCell className="hidden text-muted-foreground sm:table-cell">
                  {user.full_name ?? "—"}
                </TableCell>
                <TableCell>
                  <div className="flex flex-wrap gap-1">
                    {user.is_superuser ? <Badge>Superuser</Badge> : null}
                    {!user.is_active ? (
                      <Badge variant="secondary">Inactive</Badge>
                    ) : null}
                  </div>
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => setEditing(user)}
                    >
                      Edit
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      disabled={user.id === me.data.id}
                      onClick={() => onDelete(user)}
                    >
                      Delete
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      )}

      {count > PAGE_SIZE ? (
        <div className="flex items-center justify-end gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={skip === 0}
            onClick={() => setSkip((value) => Math.max(0, value - PAGE_SIZE))}
          >
            Previous
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={skip + PAGE_SIZE >= count}
            onClick={() => setSkip((value) => value + PAGE_SIZE)}
          >
            Next
          </Button>
        </div>
      ) : null}

      <UserDialog open={creating} onOpenChange={setCreating} user={null} />
      <UserDialog
        open={Boolean(editing)}
        onOpenChange={(open) => {
          if (!open) {
            setEditing(null);
          }
        }}
        user={editing}
      />
    </div>
  );
}
