"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { problemDetail } from "@/lib/errors";
import { updateMyPasswordMutation } from "@/lib/queries";
import { passwordSchema } from "@/lib/schemas";

const schema = z
  .object({
    current_password: z.string().min(1, "Current password is required"),
    new_password: passwordSchema,
    confirm_password: z.string(),
  })
  .refine((values) => values.new_password === values.confirm_password, {
    message: "Passwords do not match",
    path: ["confirm_password"],
  });

type Values = z.infer<typeof schema>;

export function PasswordForm() {
  const update = useMutation(updateMyPasswordMutation());
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors, isSubmitting },
  } = useForm<Values>({
    resolver: zodResolver(schema),
  });

  async function onSubmit(values: Values) {
    try {
      await update.mutateAsync({
        body: {
          current_password: values.current_password,
          new_password: values.new_password,
        },
      });
      reset();
      toast.success("Password updated. Other sessions were signed out.");
    } catch (err) {
      toast.error(problemDetail(err, "Could not update password"));
    }
  }

  return (
    <form className="max-w-md space-y-4" onSubmit={handleSubmit(onSubmit)}>
      <div className="space-y-2">
        <Label htmlFor="current_password">Current password</Label>
        <Input
          id="current_password"
          type="password"
          autoComplete="current-password"
          aria-invalid={Boolean(errors.current_password)}
          {...register("current_password")}
        />
        {errors.current_password ? (
          <p className="text-sm text-destructive">
            {errors.current_password.message}
          </p>
        ) : null}
      </div>
      <div className="space-y-2">
        <Label htmlFor="new_password">New password</Label>
        <Input
          id="new_password"
          type="password"
          autoComplete="new-password"
          aria-invalid={Boolean(errors.new_password)}
          {...register("new_password")}
        />
        {errors.new_password ? (
          <p className="text-sm text-destructive">
            {errors.new_password.message}
          </p>
        ) : null}
      </div>
      <div className="space-y-2">
        <Label htmlFor="confirm_password">Confirm new password</Label>
        <Input
          id="confirm_password"
          type="password"
          autoComplete="new-password"
          aria-invalid={Boolean(errors.confirm_password)}
          {...register("confirm_password")}
        />
        {errors.confirm_password ? (
          <p className="text-sm text-destructive">
            {errors.confirm_password.message}
          </p>
        ) : null}
      </div>
      <Button type="submit" disabled={isSubmitting}>
        {isSubmitting ? "Updating…" : "Update password"}
      </Button>
    </form>
  );
}
