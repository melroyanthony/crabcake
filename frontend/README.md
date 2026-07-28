# Frontend

Next.js App Router frontend for the API in `../backend`.

## Stack

- [Next.js](https://nextjs.org) 16 with the App Router and React 19
- [TypeScript](https://www.typescriptlang.org) 7
- [Tailwind CSS](https://tailwindcss.com) 4 and [shadcn/ui](https://ui.shadcn.com)
- [Biome](https://biomejs.dev) for lint and format (not ESLint — `typescript-eslint` does not support TypeScript 7)

## Commands

```bash
npm run dev             # http://localhost:3000
npm run lint            # biome check
npm run format          # biome format --write
npm run typecheck       # tsc --noEmit
npm run build           # production build
npm run generate-client # regenerate src/client/ from ../openapi.json
```

From the repository root:

```bash
just dev-frontend
just test-frontend
just client             # writes openapi.json, then regenerates src/client/
```

## Auth

Sessions are a BFF concern. The browser talks only to same-origin routes; tokens
never appear in JavaScript.

| Route | Purpose |
| --- | --- |
| `POST /api/auth/login` | Exchange email/password for httpOnly cookies |
| `POST /api/auth/logout` | Revoke the refresh token and clear cookies |
| `POST /api/auth/logout-everywhere` | End every session, then clear cookies |
| `POST /api/auth/refresh` | Rotate cookies from the refresh token alone |
| `GET /api/auth/me` | The signed-in user |
| `/api/v1/*` | Proxy onto the Axum API, with Bearer + refresh-on-401 |

`src/proxy.ts` redirects unauthenticated visits to `/login` for dashboard-style
routes. Set `API_URL` (see `.env.local.example`) so the BFF can reach the API.

## Layout

| Path | Purpose |
| --- | --- |
| `src/app/` | Routes and layouts |
| `src/app/api/auth/` | BFF auth route handlers |
| `src/app/api/v1/` | Same-origin proxy onto the Axum API |
| `src/components/ui/` | shadcn/ui primitives — edit freely |
| `src/components/` | App components built on top of those |
| `src/lib/auth/` | Cookie helpers and server-side API calls |
| `src/lib/api.ts` | Generated client, pointed at the same-origin BFF |
| `src/client/` | Generated OpenAPI client — never hand-edit |

The client is built by hey-api with TanStack Query options and Zod schemas.
Import query helpers from `@/client/@tanstack/react-query.gen`, for example
`readMeOptions()` or `listOptions()`.

Add more UI with:

```bash
npx shadcn@latest add <component>
```
