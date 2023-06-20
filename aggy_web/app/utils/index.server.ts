
export function readEnvVar(key: string) {
  const val = process.env[key];
  if (!val) {
    throw Error(`${key} variable was not found in environment`);
  }
  return val;
}

export const SESSION_SECRET = readEnvVar('SESSION_SECRET');
