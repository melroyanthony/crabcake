/**
 * Pulls the human-readable message out of a problem+json body thrown by the
 * generated client (`throwOnError: true` rejects with the parsed JSON).
 */
export function problemDetail(error: unknown, fallback: string): string {
  if (
    error &&
    typeof error === "object" &&
    "detail" in error &&
    typeof (error as { detail: unknown }).detail === "string"
  ) {
    return (error as { detail: string }).detail;
  }
  return fallback;
}
