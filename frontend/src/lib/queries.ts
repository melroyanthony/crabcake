/**
 * Friendly names for hey-api's collision-mangled operation ids.
 *
 * Items claimed the plain `list` / `create` / … names; users admin and uploads
 * got the numbered suffixes. Import from here in UI code so the mangling stays
 * in one place.
 */
export {
  create3Mutation as createUserMutation,
  createMutation as createItemMutation,
  delete3Mutation as deleteUserMutation,
  deleteMutation as deleteItemMutation,
  list2Options as usersListOptions,
  list2QueryKey as usersListQueryKey,
  listOptions as itemsListOptions,
  listQueryKey as itemsListQueryKey,
  readMeOptions,
  readMeQueryKey,
  recoverMutation,
  resetMutation,
  signupMutation,
  update2Mutation as updateUserMutation,
  updateMeMutation,
  updateMutation as updateItemMutation,
  updateMyPasswordMutation,
} from "@/client/@tanstack/react-query.gen";
