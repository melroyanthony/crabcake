import type { NextRequest } from "next/server";
import { NextResponse } from "next/server";

import { ACCESS_COOKIE, REFRESH_COOKIE } from "@/lib/auth/cookies";

/**
 * Gates page routes that need a session.
 *
 * Presence only: a refresh cookie without an access cookie is enough to let
 * the request through, because the first `/api/...` call will rotate. Refresh
 * is deliberately not done here — tokens are single-use, and doing it on every
 * navigation would race concurrent requests into a sign-out.
 *
 * API routes are excluded from the matcher: they answer 401 JSON rather than
 * redirecting to a login page.
 */
export function proxy(request: NextRequest) {
  const access = request.cookies.get(ACCESS_COOKIE)?.value;
  const refresh = request.cookies.get(REFRESH_COOKIE)?.value;

  if (access || refresh) {
    return NextResponse.next();
  }

  const login = new URL("/login", request.url);
  login.searchParams.set(
    "next",
    request.nextUrl.pathname + request.nextUrl.search,
  );
  return NextResponse.redirect(login);
}

export const config = {
  matcher: [
    "/dashboard/:path*",
    "/settings/:path*",
    "/items/:path*",
    "/admin/:path*",
  ],
};
