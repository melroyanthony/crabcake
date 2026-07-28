import type { NextResponse } from "next/server";

import type { TokenPair } from "@/lib/auth/types";

export const ACCESS_COOKIE = "access_token";
export const REFRESH_COOKIE = "refresh_token";

// Keep these aligned with the backend defaults (ACCESS_TOKEN_EXPIRE_MINUTES /
// REFRESH_TOKEN_EXPIRE_DAYS). They only control cookie lifetime in the browser;
// the API still decides whether a token is valid.
const ACCESS_MAX_AGE_SECONDS = 30 * 60;
const REFRESH_MAX_AGE_SECONDS = 30 * 24 * 60 * 60;

function options(maxAge: number) {
  return {
    httpOnly: true,
    // Local http:// development cannot set Secure cookies; production always does.
    secure: process.env.NODE_ENV === "production",
    sameSite: "lax" as const,
    path: "/",
    maxAge,
  };
}

export function setTokenCookies(response: NextResponse, tokens: TokenPair) {
  response.cookies.set(
    ACCESS_COOKIE,
    tokens.access_token,
    options(ACCESS_MAX_AGE_SECONDS),
  );
  response.cookies.set(
    REFRESH_COOKIE,
    tokens.refresh_token,
    options(REFRESH_MAX_AGE_SECONDS),
  );
}

export function clearTokenCookies(response: NextResponse) {
  response.cookies.set(ACCESS_COOKIE, "", options(0));
  response.cookies.set(REFRESH_COOKIE, "", options(0));
}
