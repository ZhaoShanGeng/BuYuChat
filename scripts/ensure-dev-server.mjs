import { spawn } from "node:child_process";

const DEV_URL = "http://127.0.0.1:1420";
const REQUEST_TIMEOUT_MS = 5000;

function isAbortError(error) {
  return error instanceof Error && error.name === "AbortError";
}

async function inspectExistingServer() {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const response = await fetch(DEV_URL, {
      signal: controller.signal,
      headers: {
        Accept: "text/html"
      }
    });

    const body = await readPreview(response);
    const isExpectedServer =
      response.ok &&
      (body.includes("<title>BuYu</title>") || body.includes('src="/src/main.ts"'));

    return {
      reachable: true,
      expected: isExpectedServer
    };
  } catch (error) {
    if (isAbortError(error)) {
      return {
        reachable: true,
        expected: false
      };
    }

    return {
      reachable: false,
      expected: false
    };
  } finally {
    clearTimeout(timeout);
  }
}

async function readPreview(response) {
  if (!response.body) {
    return await response.text();
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let preview = "";

  try {
    while (preview.length < 4096) {
      const { done, value } = await reader.read();
      if (done || !value) break;
      preview += decoder.decode(value, { stream: true });
      if (preview.includes("<title>BuYu</title>") || preview.includes('src="/src/main.ts"')) {
        break;
      }
    }
    preview += decoder.decode();
    return preview;
  } finally {
    await reader.cancel().catch(() => {});
  }
}

function spawnDevServer() {
  const command = process.platform === "win32" ? "pnpm dev" : "pnpm dev";
  const child = spawn(command, {
    stdio: "inherit",
    env: process.env,
    shell: true
  });

  const forwardSignal = (signal) => {
    if (!child.killed) {
      child.kill(signal);
    }
  };

  process.on("SIGINT", forwardSignal);
  process.on("SIGTERM", forwardSignal);

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    process.exit(code ?? 0);
  });
}

const existingServer = await inspectExistingServer();

if (existingServer.expected) {
  console.log(`Reusing existing BuYu dev server at ${DEV_URL}`);
  await waitForTerminationSignal();
  process.exit(0);
}

if (existingServer.reachable) {
  console.error(`Port 1420 is already occupied by a non-BuYu server: ${DEV_URL}`);
  process.exit(1);
}

spawnDevServer();

function waitForTerminationSignal() {
  return new Promise((resolve) => {
    const finish = () => resolve();
    process.once("SIGINT", finish);
    process.once("SIGTERM", finish);
  });
}
