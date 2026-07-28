"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { toast } from "sonner";

import { ItemDialog } from "@/app/(app)/items/item-dialog";
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
import type { Item } from "@/lib/api";
import { problemDetail } from "@/lib/errors";
import {
  deleteItemMutation,
  itemsListOptions,
  itemsListQueryKey,
} from "@/lib/queries";

const PAGE_SIZE = 20;

export function ItemsTable() {
  const queryClient = useQueryClient();
  const [skip, setSkip] = useState(0);
  const [editing, setEditing] = useState<Item | null>(null);
  const [creating, setCreating] = useState(false);

  const list = useQuery(
    itemsListOptions({ query: { skip, limit: PAGE_SIZE } }),
  );
  const remove = useMutation(deleteItemMutation());

  async function onDelete(item: Item) {
    if (!window.confirm(`Delete “${item.title}”?`)) {
      return;
    }
    try {
      await remove.mutateAsync({ path: { id: item.id } });
      toast.success("Item deleted");
      await queryClient.invalidateQueries({ queryKey: itemsListQueryKey() });
    } catch (err) {
      toast.error(problemDetail(err, "Could not delete item"));
    }
  }

  const count = list.data?.count ?? 0;
  const page = list.data?.data ?? [];

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <p className="text-sm text-muted-foreground">
          {list.isPending
            ? "Loading…"
            : `${count} item${count === 1 ? "" : "s"}`}
        </p>
        <Button onClick={() => setCreating(true)}>New item</Button>
      </div>

      {list.isPending ? (
        <div className="space-y-2">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-10 w-full" />
        </div>
      ) : list.error ? (
        <p className="text-sm text-destructive">Could not load items.</p>
      ) : page.length === 0 ? (
        <p className="rounded-lg border border-dashed px-4 py-10 text-center text-sm text-muted-foreground">
          No items yet. Create one to get started.
        </p>
      ) : (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Title</TableHead>
              <TableHead className="hidden sm:table-cell">
                Description
              </TableHead>
              <TableHead className="w-40 text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {page.map((item) => (
              <TableRow key={item.id}>
                <TableCell className="font-medium">{item.title}</TableCell>
                <TableCell className="hidden max-w-xs truncate text-muted-foreground sm:table-cell">
                  {item.description ?? "—"}
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-1">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => setEditing(item)}
                    >
                      Edit
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => onDelete(item)}
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

      <ItemDialog open={creating} onOpenChange={setCreating} item={null} />
      <ItemDialog
        open={Boolean(editing)}
        onOpenChange={(open) => {
          if (!open) {
            setEditing(null);
          }
        }}
        item={editing}
      />
    </div>
  );
}
