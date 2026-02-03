export const SLOT_SIZE_INT32 = 10;
export const SLOT_SIZE_BYTES = SLOT_SIZE_INT32 * 4;
export const MAX_PROCESSES = 32; // TODO: Make dynamic eventually

export const OFFSET_STATE = 0;
export const OFFSET_SYSCALL_NR = 1;
export const OFFSET_ARG0 = 2;
export const OFFSET_ARG1 = 3;
export const OFFSET_ARG2 = 4;
export const OFFSET_ARG3 = 5;
export const OFFSET_ARG4 = 6;
export const OFFSET_ARG5 = 7;
export const OFFSET_RETURN = 8;
export const OFFSET_UNUSED = 9;

export const SYS_READ = 0;
export const SYS_WRITE = 1;
export const SYS_OPEN = 2;
export const SYS_CLOSE = 3;
export const SYS_IOCTL = 16;

export function getSlotBase(pid) {
  return pid * SLOT_SIZE_INT32;
}
