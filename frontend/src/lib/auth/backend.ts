import type { TokenPair } from "@/lib/auth/types";

/**
 * Base URL of the Axum API, as seen from this Next.js process.
 *
 * In Compose that is `http://backend:8000`. When running `next dev` on a laptop
 * against a local API it is `http://localhost:8000`. Never expose this to the
 * browser — authenticated traffic goes through same-origin `/api/...` routes.
 */
export function apiUrl(path = ""): string {
  const base = process.env.API_URL;

  if (!base) {
    throw new Error(
      "API_URL is not set. Point it at the Axum API, for example http://localhost:8000",
    );
  }

  const normalised = base.replace(/\/$/, "");
  if (!path) {
    return normalised;
  }

  return `${normalised}${path.startsWith("/") ? path : `/${path}`}`;
}

/** Forwards an upstream response body and content type with its status. */
export async function passthrough(upstream: Response): Promise<Response> {
  const contentType = upstream.headers.get("content-type");
  const body = await upstream.arrayBuffer();
  const headers = new Headers();

  if (contentType) {
    headers.set("content-type", contentType);
  }

  return new Response(body, { status: upstream.status, headers });
}

/**
 * Exchanges a refresh token for a new pair.
 *
 * Returns null when the refresh is refused, so callers can clear the session
 * without caring why (expired, already used, invented).
 */
export async function exchangeRefresh(
  refreshToken: string,
): Promise<TokenPair | null> {
  const upstream = await fetch(apiUrl("/api/v1/login/refresh"), {
    method: "POST",
    headers: {
      accept: "application/json",
      "content-type": "application/json",
    },
    body: JSON.stringify({ refresh_token: refreshToken }),
    cache: "no-store",
  });

  if (!upstream.ok) {
    return null;
  }

  return (await upstream.json()) as TokenPair;
}

/**
 * Single-flight refresh for this process.
 *
 * Refresh tokens are single-use. Two concurrent 401 retries with the same
 * cookie would otherwise race: the first succeeds and the second signs the
 * user out. Sharing the in-flight promise keeps one exchange per token.
 */
let refreshInFlight: Promise<TokenPair | null> | null = null;
let refreshInFlightToken: string | null = null;

export function refreshOnce(refreshToken: string): Promise<TokenPair | null> {
  if (refreshInFlight && refreshInFlightToken === refreshToken) {
    return refreshInFlight;
  }

  refreshInFlightToken = refreshToken;
  refreshInFlight = exchangeRefresh(refreshToken).finally(() => {
    refreshInFlight = null;
    refreshInFlightToken = null;
  });

  return refreshInFlight;
}
