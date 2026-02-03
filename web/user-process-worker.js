import {
  OFFSET_STATE,
  OFFSET_SYSCALL_NR,
  OFFSET_ARG0,
  OFFSET_ARG1,
  OFFSET_ARG2,
  OFFSET_ARG3,
  OFFSET_ARG4,
  OFFSET_ARG5,
  OFFSET_RETURN,
  getSlotBase
} from './syscalls.js';

let myPid = -1;
let syscallSlots = null;
let processMemory = null;

self.onmessage = async (event) => {
  const msg = event.data;

  if (msg.type === 'init') {
    await handleInit(msg);
  } else {
    console.error('Unknown message type:', msg.type);
  }
};

async function handleInit(msg) {
  myPid = msg.pid;

  syscallSlots = new Int32Array(msg.syscallBuffer);

  processMemory = new WebAssembly.Memory({
    initial: 32,
    maximum: 256,
    shared: true
  });

  self.postMessage({
    type: 'memory_ready',
    memory: processMemory.buffer
  });

  try {
    const wasmModule = await WebAssembly.instantiate(msg.wasmBytes, {
      kernel: {
        syscall: makeSyscall
      },
      env: {
        memory: processMemory
      }
    });

    wasmModule.instance.exports._start();

    self.postMessage({
      type: 'exit',
      exitCode: 0
    });

  } catch (error) {
    console.error(`[Worker PID ${myPid}] Error:`, error);
    self.postMessage({
      type: 'exit',
      exitCode: 1,
      error: error.message
    });
  }
}

function makeSyscall(nr, a0, a1, a2, a3, a4, a5) {
  const slotBase = getSlotBase(myPid);

  Atomics.store(syscallSlots, slotBase + OFFSET_SYSCALL_NR, nr);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG0, a0);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG1, a1);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG2, a2);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG3, a3);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG4, a4);
  Atomics.store(syscallSlots, slotBase + OFFSET_ARG5, a5);

  Atomics.store(syscallSlots, slotBase + OFFSET_STATE, 1);

  const stateIndex = slotBase + OFFSET_STATE;

  while (Atomics.load(syscallSlots, stateIndex) !== 2) {
    const result = Atomics.wait(syscallSlots, stateIndex, 1, -1);

    if (result === 'not-equal') {
      break;
    }
  }

  const returnValue = Atomics.load(syscallSlots, slotBase + OFFSET_RETURN);

  Atomics.store(syscallSlots, slotBase + OFFSET_STATE, 0);

  return returnValue;
}
