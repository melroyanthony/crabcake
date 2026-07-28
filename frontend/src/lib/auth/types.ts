/** Shapes the BFF speaks before the generated OpenAPI client exists. */

export type TokenPair = {
  access_token: string;
  refresh_token: string;
  token_type: string;
};

export type UserPublic = {
  id: string;
  email: string;
  full_name: string | null;
  is_active: boolean;
  is_superuser: boolean;
  created_at: string;
};

export type Problem = {
  status: number;
  title: string;
  detail: string;
};
