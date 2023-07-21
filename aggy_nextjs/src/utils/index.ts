export { SessionStore } from "./session";

export function dbg<T>(value: T): T {
  console.log("DBG: ", value);
  return value;
}

export function assertNotNull<T>(value: T | undefined): T {
  if (value === undefined) {
    throw Error('Value was null');
  }
  return value;
}
