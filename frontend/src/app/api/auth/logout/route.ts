import { NextResponse } from "next/server";

import { apiUrl } from "@/lib/auth/backend";
import { clearTokenCookies } from "@/lib/auth/cookies";
import { getRefreshToken } from "@/lib/auth/session";

/**
 * Ends the current session.
 *
 * Always clears the cookies, even when there is nothing to revoke upstream —
 * signing out must not fail because a cookie was already gone.
 */
export async function POST() {
  const refreshToken = await getRefreshToken();

  if (refreshToken) {
    await fetch(apiUrl("/api/v1/login/logout"), {
      method: "POST",
      headers: {
        accept: "application/json",
        "content-type": "application/json",
      },
      body: JSON.stringify({ refresh_token: refreshToken }),
      cache: "no-store",
    }).catch(() => {
      // The cookies still have to go even if the API is unreachable.
    });
  }

  const response = NextResponse.json({ message: "signed out" });
  clearTokenCookies(response);
  return response;
}
