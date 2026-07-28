import type { APIRequestContext } from "@playwright/test";

import { mailcatcherHost } from "../config";

type Email = {
  id: number;
  recipients: string[];
  subject: string;
};

async function listEmails(request: APIRequestContext): Promise<Email[]> {
  const response = await request.get(`${mailcatcherHost}/messages`);
  if (!response.ok()) {
    throw new Error(`Mailcatcher list failed: ${response.status()}`);
  }
  return (await response.json()) as Email[];
}

async function findEmail(
  request: APIRequestContext,
  filter?: (email: Email) => boolean,
): Promise<Email | null> {
  let emails = await listEmails(request);
  if (filter) {
    emails = emails.filter(filter);
  }
  return emails.at(-1) ?? null;
}

/** Polls Mailcatcher until a matching message appears or the timeout elapses. */
export async function findLastEmail({
  request,
  filter,
  timeout = 15_000,
}: {
  request: APIRequestContext;
  filter?: (email: Email) => boolean;
  timeout?: number;
}): Promise<Email> {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const email = await findEmail(request, filter);
    if (email) {
      return email;
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error("Timed out waiting for Mailcatcher message");
}

/** Reads the plain-text body and returns the reset-password link. */
export async function resetPasswordUrl(
  request: APIRequestContext,
  emailId: number,
): Promise<string> {
  // Plain text avoids quoted-printable soft line breaks that shred hrefs in HTML.
  const response = await request.get(
    `${mailcatcherHost}/messages/${emailId}.plain`,
  );
  if (!response.ok()) {
    throw new Error(`Mailcatcher message failed: ${response.status()}`);
  }
  const text = await response.text();
  const match = text.match(/https?:\/\/\S+\/reset-password\?token=\S+/);
  if (!match) {
    throw new Error("Reset link not found in email body");
  }
  return match[0].replace(/[>\s]+$/g, "").replace(/&amp;/g, "&");
}

export function recipientMatches(email: Email, address: string) {
  const needle = address.toLowerCase();
  return email.recipients.some((recipient) =>
    recipient.toLowerCase().includes(needle),
  );
}
