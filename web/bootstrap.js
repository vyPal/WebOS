const logEl = document.getElementById("log");

function log(msg) {
  logEl.textContent += msg + "\n";
}

const memory = new WebAssembly.Memory({
  initial: 32,
});
var kernelInstance;

function readMemory(ptr, len) {
  return new Uint8Array(memory.buffer, ptr, len);
}

const imports = {
  sys: {
    os_write(fd, ptr, len) {
      log(new TextDecoder().decode(readMemory(ptr, len)));
      return 0;
    }
  },
  env: {
    memory: memory
  }
};

async function loadWasm(path, pid) {
  const resp = await fetch(path);
  const bytes = await resp.arrayBuffer();
  let customImports = {
    ...imports,
    env: { ...imports.env, pid: pid }
  };

  if (kernelInstance) {
    customImports.kernel = {
      syscall(nr, a0, a1, a2, a3, a4, a5) {
        return kernelInstance.instance.exports.syscall(pid, nr, a0, a1, a2, a3, a4, a5);
      }
    };
  }

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

