import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Produces a self-contained server folder for the Docker image instead of
  // shipping a full `node_modules` tree at runtime.
  output: "standalone",
  // TypeScript 7's package no longer ships the JavaScript compiler API Next.js
  // used to load. This runs the project-local `tsc` instead, which is what makes
  // `next build` work on typescript@7.
  experimental: {
    useTypeScriptCli: true,
  },
};

export default nextConfig;
