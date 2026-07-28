import { NextResponse } from "next/server";

import { apiUrl, refreshOnce } from "@/lib/auth/backend";
import { clearTokenCookies, setTokenCookies } from "@/lib/auth/cookies";
import { getAccessToken, getRefreshToken } from "@/lib/auth/session";
import type { TokenPair } from "@/lib/auth/types";

/**
 * The signed-in user, or 401.
 *
 * Refreshes once when the access cookie has expired so a page load after a
 * coffee break still knows who is there.
 */
export async function GET() {
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

  let upstream = await fetchMe(accessToken);

  if (upstream.status === 401 && refreshToken && !rotated) {
    rotated = await refreshOnce(refreshToken);
    if (rotated) {
      accessToken = rotated.access_token;
      upstream = await fetchMe(accessToken);
    }
  }

  if (!upstream.ok) {
    const response = new NextResponse(upstream.body, {
      status: upstream.status,
      headers: contentTypeHeaders(upstream),
    });
    if (upstream.status === 401 || upstream.status === 403) {
      clearTokenCookies(response);
    } else if (rotated) {
      setTokenCookies(response, rotated);
    }
    return response;
  }

  const response = new NextResponse(upstream.body, {
    status: upstream.status,
    headers: contentTypeHeaders(upstream),
  });
  if (rotated) {
    setTokenCookies(response, rotated);
  }
  return response;
}

async function fetchMe(accessToken: string): Promise<Response> {
  return fetch(apiUrl("/api/v1/users/me"), {
    headers: {
      accept: "application/json",
      authorization: `Bearer ${accessToken}`,
    },
    cache: "no-store",
  });
}

function contentTypeHeaders(upstream: Response): Headers {
  const headers = new Headers();
  const contentType = upstream.headers.get("content-type");
  if (contentType) {
    headers.set("content-type", contentType);
  }
  return headers;
}
