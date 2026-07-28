import { z } from "zod";

/** Matches the backend `Password` validator (8–128 characters). */
export const passwordSchema = z
  .string()
  .min(8, "Password must be at least 8 characters")
  .max(128, "Password must be at most 128 characters");

export const emailSchema = z.email("Enter a valid email address");
