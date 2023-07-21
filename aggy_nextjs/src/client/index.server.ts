
import { AggyClient, ApiClient, T } from "./"
import { SessionStore, assertNotNull } from '@/utils';

export function apiClient() {
  const aggy = new AggyClient(AGGY_SERVICE_SECRET, AGGY_BASE_URL);
  return {
    client: new ApiClient(
      aggy
    ),
    session: new SessionStore(aggy)
  };
}

/* export function readEnvVar(key: string) {
  const val = process.env[key];
  if (!val) {
    throw Error(`${key} variable was not found in environment: ${JSON.stringify(process.env, null, ' ')}`);
  }
  return val;
} */

const AGGY_BASE_URL = assertNotNull(process.env.AGGY_BASE_URL);
const AGGY_SERVICE_SECRET = assertNotNull(process.env.AGGY_SERVICE_SECRET);

