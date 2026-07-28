import { NextResponse } from "next/server";

import { apiUrl, refreshOnce } from "@/lib/auth/backend";
import { clearTokenCookies, setTokenCookies } from "@/lib/auth/cookies";
import { getAccessToken, getRefreshToken } from "@/lib/auth/session";
import type { TokenPair } from "@/lib/auth/types";

/**
 * Ends every session the caller has.
 *
 * Needs a live access token. When only the refresh cookie remains, one refresh
 * is attempted first so this still works after a long idle.
 */
export async function POST() {
  let accessToken = await getAccessToken();
  const refreshToken = await getRefreshToken();
  let rotated: TokenPair | null = null;

  if (!accessToken && refreshToken) {
    rotated = await refreshOnce(refreshToken);
    accessToken = rotated?.access_token;
  }

  if (!accessToken) {
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

  const upstream = await fetch(apiUrl("/api/v1/login/logout-everywhere"), {
    method: "POST",
    headers: {
      accept: "application/json",
      authorization: `Bearer ${accessToken}`,
    },
    cache: "no-store",
  });

  if (!upstream.ok) {
    const response = new NextResponse(upstream.body, {
      status: upstream.status,
      headers: contentTypeHeaders(upstream),
    });

    if (upstream.status === 401 || upstream.status === 403) {
      clearTokenCookies(response);
    } else if (rotated) {
      // Keep the rotated cookies so a retry can use them rather than leaving
      // the browser holding a spent refresh token.
      setTokenCookies(response, rotated);
    }

    return response;
  }

  const response = NextResponse.json({ message: "signed out everywhere" });
  clearTokenCookies(response);
  return response;
}

function contentTypeHeaders(upstream: Response): Headers {
  const headers = new Headers();
  const contentType = upstream.headers.get("content-type");
  if (contentType) {
    headers.set("content-type", contentType);
  }
  return headers;
}
