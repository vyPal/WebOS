const logEl = document.getElementById("log");

function log(msg) {
  logEl.textContent += msg + "\n";
}

const memory = new WebAssembly.Memory({
  initial: 32,
});
var kernelInstance;

function readMemory(ptr, len) {
  return new Uint8Array(memories[0].buffer, ptr, len);
}

const memories = [];

async function loadWasm(path, pid) {
  if (pid < memories.length) return new Promise.reject("PID already allocated")

  const resp = await fetch(path);
  const bytes = await resp.arrayBuffer();

  memories.push(new WebAssembly.Memory({
    initial: 32,
  }));

  // Different imports basedon if process is kernel (pid == 0) or not
  let customImports = (pid == 0) ? {
    sys: {
      serial_write(ptr, len) {
        log(new TextDecoder().decode(readMemory(ptr, len)));
        return 0;
      }
    },
    mem_ops: {
      cp_from_bin(otherpid, otherptr, ptr, len) {
        new Uint8Array(memories[pid].buffer, ptr, len).set(new Uint8Array(memories[otherpid].buffer, otherptr, len))
      },
      cp_to_bin(otherpid, otherptr, ptr, len) {
        new Uint8Array(memories[otherpid].buffer, otherptr, len).set(new Uint8Array(memories[pid].buffer, ptr, len))
      },
    },
    env: {
      memory: memories[pid]
    }
  } : {
    kernel: {
      syscall(nr, a0, a1, a2, a3, a4, a5) {
        return kernelInstance.instance.exports.syscall(pid, nr, a0, a1, a2, a3, a4, a5);
      }
    },
    env: {
      pid: pid,
      memory: memories[pid]
    }
  };

  let inst = await WebAssembly.instantiate(bytes, customImports);
  inst.instance.exports._start();
  return inst
}

async function main() {
  log("boot: loading kernel");
  console.log(kernelInstance)
  kernelInstance = await loadWasm("/kernel.wasm", 0);
  console.log(kernelInstance)

  log("boot: loading hello");
  await loadWasm("/apps/hello.wasm", 1);

  log("boot: done");
}

main().catch(err => {
  console.error(err);
  log("boot error: " + err);
});

