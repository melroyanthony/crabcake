import { NextResponse } from "next/server";

import { refreshOnce } from "@/lib/auth/backend";
import { clearTokenCookies, setTokenCookies } from "@/lib/auth/cookies";
import { getRefreshToken } from "@/lib/auth/session";

/**
 * Rotates the session cookies from the refresh token alone.
 *
 * Most callers never need this: the `/api/v1` proxy refreshes on 401. It exists
 * so a page load that finds an expired access cookie can renew without waiting
 * for the first API call to fail.
 */
export async function POST() {
  const refreshToken = await getRefreshToken();

  if (!refreshToken) {
    const response = NextResponse.json(
      {
        status: 401,
        title: "Unauthorized",
        detail: "not authenticated",
      },
      { status: 401 },
    );
    clearTokenCookies(response);
    return response;
  }

  const tokens = await refreshOnce(refreshToken);

  if (!tokens) {
    const response = NextResponse.json(
      {
        status: 401,
        title: "Unauthorized",
        detail: "session expired",
      },
      { status: 401 },
    );
    clearTokenCookies(response);
    return response;
  }

  const response = NextResponse.json({ ok: true });
  setTokenCookies(response, tokens);
  return response;
}
