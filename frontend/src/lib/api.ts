import { client } from "@/client/client.gen";

/**
 * Points the generated SDK at the same-origin BFF proxy.
 *
 * An empty base URL means paths like `/api/v1/items` stay on this Next.js
 * origin, so cookies are attached and refreshed by `/api/v1/[...path]` rather
 * than talking to Axum from the browser.
 */
client.setConfig({
  baseUrl: "",
});

export { client };
export type {
  Item,
  ItemCreate,
  ItemUpdate,
  PageItem,
  PageUserPublic,
  Problem,
  UserPublic,
  UserRegister,
  UserUpdateMe,
} from "@/client";
export {
  create,
  delete_,
  list,
  read,
  readMe,
  signup,
  update,
  updateMe,
  updateMyPassword,
} from "@/client";
