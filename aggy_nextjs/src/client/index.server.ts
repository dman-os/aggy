
import { AggyClient, EpigramClient, ApiClient, } from "./"
import { assertNotNull } from '@/utils';
import { SessionStore } from "@/utils/index.server";

export function apiClient() {
  const aggy = new AggyClient(AGGY_SERVICE_SECRET, AGGY_BASE_URL);
  const epigram = new EpigramClient(EPIGRAM_SERVICE_SECRET, EPIGRAM_BASE_URL);
  return {
    client: new ApiClient(
      aggy, epigram
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
const EPIGRAM_BASE_URL = assertNotNull(process.env.EPIGRAM_BASE_URL);
const EPIGRAM_SERVICE_SECRET = assertNotNull(process.env.EPIGRAM_SERVICE_SECRET);

