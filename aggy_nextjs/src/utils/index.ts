export function dbg<T>(value: T): T {
  console.log("DBG: ", value);
  return value;
}

export function assertNotNull<T>(value: T | undefined | null): T {
  if (value === undefined) {
    throw Error('Value was undefined');
  }
  if (value === null) {
    throw Error('Value was null');
  }
  return value;
}
