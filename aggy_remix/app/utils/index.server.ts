
export function readEnvVar(key: string) {
  const val = process.env[key];
  if (!val) {
    throw Error(`${key} variable was not found in environment: ${JSON.stringify(process.env, null, ' ')}`);
  }
  return val;
}

export const SESSION_SECRET = readEnvVar('SESSION_SECRET');
