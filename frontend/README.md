# Frontend

Next.js App Router frontend for the API in `../backend`.

## Stack

- [Next.js](https://nextjs.org) 16 with the App Router and React 19
- [TypeScript](https://www.typescriptlang.org) 7
- [Tailwind CSS](https://tailwindcss.com) 4 and [shadcn/ui](https://ui.shadcn.com)
- [Biome](https://biomejs.dev) for lint and format (not ESLint — `typescript-eslint` does not support TypeScript 7)

## Commands

```bash
npm run dev           # http://localhost:3000
npm run lint          # biome check
npm run format        # biome format --write
npm run typecheck     # tsc --noEmit
npm run build         # production build
```

From the repository root:

```bash
just dev-frontend
just test-frontend
just client           # regenerate src/client/ from the OpenAPI spec (once wired)
```

## Layout

| Path | Purpose |
| --- | --- |
| `src/app/` | Routes and layouts |
| `src/components/ui/` | shadcn/ui primitives — edit freely |
| `src/components/` | App components built on top of those |
| `src/lib/` | Shared helpers (`cn`, and later the API helpers) |
| `src/client/` | Generated OpenAPI client — never hand-edit |

Add more UI with:

```bash
npx shadcn@latest add <component>
```
