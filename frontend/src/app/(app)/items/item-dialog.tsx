"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { Item } from "@/lib/api";
import { problemDetail } from "@/lib/errors";
import {
  createItemMutation,
  itemsListQueryKey,
  updateItemMutation,
} from "@/lib/queries";

const schema = z.object({
  title: z.string().min(1, "Title is required").max(255),
  description: z.string().max(4096).optional(),
});

type Values = z.infer<typeof schema>;

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  item?: Item | null;
};

export function ItemDialog({ open, onOpenChange, item }: Props) {
  const queryClient = useQueryClient();
  const create = useMutation(createItemMutation());
  const update = useMutation(updateItemMutation());
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<Values>({
    resolver: zodResolver(schema),
    defaultValues: { title: "", description: "" },
  });

  useEffect(() => {
    if (open) {
      reset({
        title: item?.title ?? "",
        description: item?.description ?? "",
      });
    }
  }, [open, item, reset]);

  async function onSubmit(values: Values) {
    try {
      if (item) {
        await update.mutateAsync({
          path: { id: item.id },
          body: {
            title: values.title,
            description: values.description || null,
          },
        });
        toast.success("Item updated");
      } else {
        await create.mutateAsync({
          body: {
            title: values.title,
            description: values.description || null,
          },
        });
        toast.success("Item created");
      }
      await queryClient.invalidateQueries({ queryKey: itemsListQueryKey() });
      onOpenChange(false);
    } catch (err) {
      toast.error(problemDetail(err, "Could not save item"));
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <DialogHeader>
            <DialogTitle>{item ? "Edit item" : "New item"}</DialogTitle>
            <DialogDescription>
              Items belong to you. Superusers can see everyone’s.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="title">Title</Label>
            <Input
              id="title"
              aria-invalid={Boolean(errors.title)}
              {...register("title")}
            />
            {errors.title ? (
              <p className="text-sm text-destructive">{errors.title.message}</p>
            ) : null}
          </div>
          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea id="description" rows={3} {...register("description")} />
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Saving…" : "Save"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
