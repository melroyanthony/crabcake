import { loadRootEnv } from "./load-env";

loadRootEnv();

export const firstSuperuser =
  process.env.FIRST_SUPERUSER ?? "admin@example.com";
export const firstSuperuserPassword =
  process.env.FIRST_SUPERUSER_PASSWORD ?? "changethis";

export const mailcatcherHost =
  process.env.MAILCATCHER_HOST ?? "http://localhost:1080";
