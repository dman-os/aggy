import { AggyClient, ApiClient } from "./"

export function readEnvVar(key: string) {
  const val = process.env[key];
  if (!val) {
    throw Error(`${key} variable was not found in environment: ${JSON.stringify(process.env, null, ' ')}`);
  }
  return val;
}

const AGGY_BASE_URL = readEnvVar("AGGY_BASE_URL");

export function apiClient() {
  return {
    client: new ApiClient(
      new AggyClient(AGGY_BASE_URL)
    )
  };
}
