import {
  SLOT_SIZE_BYTES,
  MAX_PROCESSES,
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

const logEl = document.getElementById("log");

function log(msg) {
  logEl.textContent += msg + "\n";
}

let kernelInstance;
let kernelMemory;
const syscallBuffer = new SharedArrayBuffer(SLOT_SIZE_BYTES * MAX_PROCESSES);
const syscallSlots = new Int32Array(syscallBuffer);
const workers = new Map();
const processData = new Map();
let nextPid = 1;

async function main() {
  await initKernel();

  startSyscallLoop();

  const helloBytes = await fetchWasm('/apps/hello.wasm');
  await spawnProcess(helloBytes);
}

async function initKernel() {
  const kernelBytes = await fetchWasm('/kernel.wasm');

  kernelMemory = new WebAssembly.Memory({
    initial: 32,
    maximum: 256,
    shared: false
  });

  const result = await WebAssembly.instantiate(kernelBytes, {
    sys: {
      serial_write(ptr, len) {
        const buffer = new Uint8Array(kernelMemory.buffer, ptr, len);
        const text = new TextDecoder().decode(buffer);
        log(text.trimEnd());
        return len;
      }
    },
    mem_ops: {
      cp_from_bin(pid, processPtr, kernelPtr, len) {
        const procData = processData.get(pid);
        if (!procData) {
          console.error(`cp_from_bin: unknown PID ${pid}`);
          return;
        }

        const kernelBuf = new Uint8Array(kernelMemory.buffer);
        const processBuf = procData.memory;

        for (let i = 0; i < len; i++) {
          kernelBuf[kernelPtr + i] = processBuf[processPtr + i];
        }
      },

      cp_to_bin(pid, processPtr, kernelPtr, len) {
        const procData = processData.get(pid);
        if (!procData) {
          console.error(`cp_to_bin: unknown PID ${pid}`);
          return;
        }

        const kernelBuf = new Uint8Array(kernelMemory.buffer);
        const processBuf = procData.memory;

        for (let i = 0; i < len; i++) {
          processBuf[processPtr + i] = kernelBuf[kernelPtr + i];
        }
      }
    },
    env: {
      memory: kernelMemory
    }
  });

  kernelInstance = result.instance;

  kernelInstance.exports._start();
}

async function fetchWasm(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
  }
  return await response.arrayBuffer();
}

async function spawnProcess(wasmBytes) {
  const pid = nextPid++;

  const worker = new Worker('user-process-worker.js', { type: 'module' });
  workers.set(pid, worker);

  const memoryPromise = new Promise((resolve) => {
    const handler = (event) => {
      if (event.data.type === 'memory_ready') {
        worker.removeEventListener('message', handler);
        resolve(event.data.memory);
      }
    };
    worker.addEventListener('message', handler);
  });

  worker.addEventListener('message', (event) => {
    if (event.data.type === 'exit') {
      workers.delete(pid);
      processData.delete(pid);
    }
  });

  worker.postMessage({
    type: 'init',
    pid: pid,
    syscallBuffer: syscallBuffer,
    wasmBytes: wasmBytes
  });

  const memoryBuffer = await memoryPromise;

  processData.set(pid, {
    memory: new Uint8Array(memoryBuffer),
    state: 'running',
  });
}

function startSyscallLoop() {
  function tick() {
    for (let pid = 1; pid < MAX_PROCESSES; pid++) {
      if (!processData.has(pid)) {
        continue;
      }

      const slotBase = getSlotBase(pid);
      const state = Atomics.load(syscallSlots, slotBase + OFFSET_STATE);

      if (state === 1) {
        handleSyscall(pid, slotBase);
      }
    }

    setTimeout(tick, 0);
  }

  tick();
}

function handleSyscall(pid, slotBase) {
  const nr = Atomics.load(syscallSlots, slotBase + OFFSET_SYSCALL_NR);
  const a0 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG0);
  const a1 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG1);
  const a2 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG2);
  const a3 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG3);
  const a4 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG4);
  const a5 = Atomics.load(syscallSlots, slotBase + OFFSET_ARG5);

  const result = kernelInstance.exports.syscall(pid, nr, a0, a1, a2, a3, a4, a5);

  Atomics.store(syscallSlots, slotBase + OFFSET_RETURN, result);

  Atomics.store(syscallSlots, slotBase + OFFSET_STATE, 2);
  Atomics.notify(syscallSlots, slotBase + OFFSET_STATE, 1);
}

main().catch(err => {
  log("boot error: " + err);
  console.error("Boot error:", err);
});

