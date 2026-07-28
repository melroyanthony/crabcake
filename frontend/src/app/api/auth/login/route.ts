import { NextResponse } from "next/server";

import { apiUrl, passthrough } from "@/lib/auth/backend";
import { setTokenCookies } from "@/lib/auth/cookies";
import type { TokenPair } from "@/lib/auth/types";

/**
 * Signs in through the BFF.
 *
 * Tokens never reach the browser as JSON: they are written into httpOnly
 * cookies and the response is a bare acknowledgement.
 */
export async function POST(request: Request) {
  let body: unknown;

  try {
    body = await request.json();
  } catch {
    return NextResponse.json(
      {
        status: 422,
        title: "Unprocessable Entity",
        detail: "request body must be JSON",
      },
      { status: 422 },
    );
  }

  const upstream = await fetch(apiUrl("/api/v1/login/access-token"), {
    method: "POST",
    headers: {
      accept: "application/json",
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
    cache: "no-store",
  });

  if (!upstream.ok) {
    return passthrough(upstream);
  }

  const tokens = (await upstream.json()) as TokenPair;
  const response = NextResponse.json({ ok: true });
  setTokenCookies(response, tokens);
  return response;
}
