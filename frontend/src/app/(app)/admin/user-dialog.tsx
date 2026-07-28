"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
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
import type { UserPublic } from "@/lib/api";
import { problemDetail } from "@/lib/errors";
import {
  createUserMutation,
  updateUserMutation,
  usersListQueryKey,
} from "@/lib/queries";
import { emailSchema, passwordSchema } from "@/lib/schemas";

const createSchema = z.object({
  email: emailSchema,
  full_name: z.string().max(255).optional(),
  password: passwordSchema,
  is_active: z.boolean(),
  is_superuser: z.boolean(),
});

const editSchema = z.object({
  email: emailSchema,
  full_name: z.string().max(255).optional(),
  password: z
    .string()
    .optional()
    .refine(
      (value) => !value || (value.length >= 8 && value.length <= 128),
      "Password must be 8–128 characters when set",
    ),
  is_active: z.boolean(),
  is_superuser: z.boolean(),
});

type CreateValues = z.infer<typeof createSchema>;
type EditValues = z.infer<typeof editSchema>;

type Props = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  user?: UserPublic | null;
};

export function UserDialog({ open, onOpenChange, user }: Props) {
  const queryClient = useQueryClient();
  const create = useMutation(createUserMutation());
  const update = useMutation(updateUserMutation());
  const editing = Boolean(user);

  const form = useForm<CreateValues | EditValues>({
    resolver: zodResolver(editing ? editSchema : createSchema),
    defaultValues: {
      email: "",
      full_name: "",
      password: "",
      is_active: true,
      is_superuser: false,
    },
  });

  const {
    register,
    handleSubmit,
    control,
    reset,
    formState: { errors, isSubmitting },
  } = form;

  useEffect(() => {
    if (open) {
      reset({
        email: user?.email ?? "",
        full_name: user?.full_name ?? "",
        password: "",
        is_active: user?.is_active ?? true,
        is_superuser: user?.is_superuser ?? false,
      });
    }
  }, [open, user, reset]);

  async function onSubmit(values: CreateValues | EditValues) {
    try {
      if (user) {
        const body = values as EditValues;
        await update.mutateAsync({
          path: { id: user.id },
          body: {
            email: body.email,
            full_name: body.full_name || null,
            is_active: body.is_active,
            is_superuser: body.is_superuser,
            password: body.password || null,
          },
        });
        toast.success("User updated");
      } else {
        const body = values as CreateValues;
        await create.mutateAsync({
          body: {
            email: body.email,
            password: body.password,
            full_name: body.full_name || null,
            is_active: body.is_active,
            is_superuser: body.is_superuser,
          },
        });
        toast.success("User created");
      }
      await queryClient.invalidateQueries({ queryKey: usersListQueryKey() });
      onOpenChange(false);
    } catch (err) {
      toast.error(problemDetail(err, "Could not save user"));
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
          <DialogHeader>
            <DialogTitle>{user ? "Edit user" : "New user"}</DialogTitle>
            <DialogDescription>
              Superuser-only. Setting a password ends that user’s sessions.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <Input
              id="email"
              type="email"
              aria-invalid={Boolean(errors.email)}
              {...register("email")}
            />
            {errors.email ? (
              <p className="text-sm text-destructive">{errors.email.message}</p>
            ) : null}
          </div>
          <div className="space-y-2">
            <Label htmlFor="full_name">Full name</Label>
            <Input id="full_name" type="text" {...register("full_name")} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">
              {user ? "New password (optional)" : "Password"}
            </Label>
            <Input
              id="password"
              type="password"
              autoComplete="new-password"
              aria-invalid={Boolean(errors.password)}
              {...register("password")}
            />
            {errors.password ? (
              <p className="text-sm text-destructive">
                {errors.password.message}
              </p>
            ) : null}
          </div>
          <div className="flex flex-col gap-3">
            <Controller
              name="is_active"
              control={control}
              render={({ field }) => (
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="is_active"
                    checked={field.value}
                    onCheckedChange={(checked) =>
                      field.onChange(checked === true)
                    }
                  />
                  <Label htmlFor="is_active">Active</Label>
                </div>
              )}
            />
            <Controller
              name="is_superuser"
              control={control}
              render={({ field }) => (
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="is_superuser"
                    checked={field.value}
                    onCheckedChange={(checked) =>
                      field.onChange(checked === true)
                    }
                  />
                  <Label htmlFor="is_superuser">Superuser</Label>
                </div>
              )}
            />
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
