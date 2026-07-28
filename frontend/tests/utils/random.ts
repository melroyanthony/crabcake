export function randomSuffix() {
  return `${Date.now().toString(36)}${Math.random().toString(36).slice(2, 8)}`;
}

export function randomEmail() {
  return `user.${randomSuffix()}@example.com`;
}

export function randomPassword() {
  return `Pw-${randomSuffix()}!`;
}

export function randomTitle() {
  return `Item ${randomSuffix()}`;
}
