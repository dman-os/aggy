
import { headers } from 'next/headers'

export function getCsrfToken() {
  const token = headers().get('X-CSRF-TOKEN');
  if (!token) {
    throw Error("No X-CSRF-TOKEN header found");
  }
  return token;
}
