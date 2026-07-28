"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { problemDetail } from "@/lib/errors";
import { readMeOptions, readMeQueryKey, updateMeMutation } from "@/lib/queries";
import { emailSchema } from "@/lib/schemas";

const schema = z.object({
  email: emailSchema,
  full_name: z.string().max(255).optional(),
});

type Values = z.infer<typeof schema>;

export function ProfileForm() {
  const queryClient = useQueryClient();
  const me = useQuery(readMeOptions());
  const update = useMutation(updateMeMutation());
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting, isDirty },
  } = useForm<Values>({
    resolver: zodResolver(schema),
  });

  useEffect(() => {
    if (me.data) {
      reset({
        email: me.data.email,
        full_name: me.data.full_name ?? "",
      });
    }
  }, [me.data, reset]);

  async function onSubmit(values: Values) {
    try {
      await update.mutateAsync({
        body: {
          email: values.email,
          full_name: values.full_name || null,
        },
      });
      await queryClient.invalidateQueries({ queryKey: readMeQueryKey() });
      toast.success("Profile updated");
    } catch (err) {
      toast.error(problemDetail(err, "Could not update profile"));
    }
  }

  if (me.isPending) {
    return (
      <div className="space-y-3">
        <Skeleton className="h-8 w-full" />
        <Skeleton className="h-8 w-full" />
      </div>
    );
  }

  if (me.error || !me.data) {
    return <p className="text-sm text-destructive">Could not load profile.</p>;
  }

  return (
    <form className="max-w-md space-y-4" onSubmit={handleSubmit(onSubmit)}>
      <div className="space-y-2">
        <Label htmlFor="email">Email</Label>
        <Input
          id="email"
          type="email"
          autoComplete="email"
          aria-invalid={Boolean(errors.email)}
          {...register("email")}
        />
        {errors.email ? (
          <p className="text-sm text-destructive">{errors.email.message}</p>
        ) : null}
      </div>
      <div className="space-y-2">
        <Label htmlFor="full_name">Full name</Label>
        <Input
          id="full_name"
          type="text"
          autoComplete="name"
          {...register("full_name")}
        />
      </div>
      <Button type="submit" disabled={isSubmitting || !isDirty}>
        {isSubmitting ? "Saving…" : "Save profile"}
      </Button>
    </form>
  );
}
