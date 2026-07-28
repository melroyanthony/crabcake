import { NextResponse } from "next/server";

import { apiUrl, refreshOnce } from "@/lib/auth/backend";
import { clearTokenCookies, setTokenCookies } from "@/lib/auth/cookies";
import { getAccessToken, getRefreshToken } from "@/lib/auth/session";
import type { TokenPair } from "@/lib/auth/types";

type RouteContext = {
  params: Promise<{ path: string[] }>;
};

/**
 * Same-origin proxy onto the Axum API.
 *
 * The browser never sees the backend URL or the tokens. Cookies become a
 * Bearer header here; on 401 a single refresh is attempted and the original
 * request is retried with the new access token.
 *
 * Token-issuing login routes are refused: those go through `/api/auth/*` so
 * tokens land in cookies rather than in a JSON body the browser can read.
 */
async function proxy(request: Request, context: RouteContext) {
  const { path } = await context.params;
  const joined = path.join("/");

  if (isReserved(joined)) {
    return NextResponse.json(
      {
        status: 404,
        title: "Not Found",
        detail: "use /api/auth for sign-in and sign-out",
      },
      { status: 404 },
    );
  }

  const url = new URL(request.url);
  const target = apiUrl(`/api/v1/${joined}${url.search}`);
  const body =
    request.method === "GET" || request.method === "HEAD"
      ? undefined
      : await request.arrayBuffer();

  let accessToken = await getAccessToken();
  const refreshToken = await getRefreshToken();
  let rotated: TokenPair | null = null;

  let upstream = await forward(request, target, accessToken, body);

  if (upstream.status === 401 && refreshToken) {
    rotated = await refreshOnce(refreshToken);
    if (rotated) {
      accessToken = rotated.access_token;
      upstream = await forward(request, target, accessToken, body);
    } else {
      const response = new NextResponse(upstream.body, {
        status: 401,
        headers: contentTypeHeaders(upstream),
      });
      clearTokenCookies(response);
      return response;
    }
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

async function forward(
  request: Request,
  target: string,
  accessToken: string | undefined,
  body: ArrayBuffer | undefined,
): Promise<Response> {
  const headers = new Headers();
  const contentType = request.headers.get("content-type");
  const accept = request.headers.get("accept");

  if (contentType) {
    headers.set("content-type", contentType);
  }
  if (accept) {
    headers.set("accept", accept);
  }
  if (accessToken) {
    headers.set("authorization", `Bearer ${accessToken}`);
  }

  return fetch(target, {
    method: request.method,
    headers,
    body,
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

function isReserved(path: string): boolean {
  return (
    path === "login/access-token" ||
    path === "login/refresh" ||
    path === "login/logout"
  );
}

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const PATCH = proxy;
export const DELETE = proxy;
