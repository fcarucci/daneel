#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0


import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import http from "node:http";
import process from "node:process";

function parseArgs(argv) {
  const options = {
    url: "",
    screenshot: "",
    dom: "",
    waitTexts: [],
    forbidTexts: [],
    timeoutMs: 40_000,
    viewport: "1600,1400",
    debugPort: 9229,
    chromePath: "google-chrome",
    fullPage: false,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    const next = argv[i + 1];

    if (arg === "--url") {
      options.url = next;
      i += 1;
    } else if (arg === "--screenshot") {
      options.screenshot = next;
      i += 1;
    } else if (arg === "--dom") {
      options.dom = next;
      i += 1;
    } else if (arg === "--wait-text") {
      options.waitTexts.push(next);
      i += 1;
    } else if (arg === "--forbid-text") {
      options.forbidTexts.push(next);
      i += 1;
    } else if (arg === "--timeout-ms") {
      options.timeoutMs = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--viewport") {
      options.viewport = next;
      i += 1;
    } else if (arg === "--debug-port") {
      options.debugPort = Number.parseInt(next, 10);
      i += 1;
    } else if (arg === "--chrome-path") {
      options.chromePath = next;
      i += 1;
    } else if (arg === "--full-page") {
      options.fullPage = true;
    }
  }

  if (!options.url) throw new Error("Missing required --url");
  if (!options.screenshot) throw new Error("Missing required --screenshot");
  if (options.waitTexts.length === 0) {
    throw new Error("Provide at least one --wait-text");
  }

  return options;
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function httpJson(url) {
  return new Promise((resolve, reject) => {
    http
      .get(url, (response) => {
        let raw = "";
        response.setEncoding("utf8");
        response.on("data", (chunk) => {
          raw += chunk;
        });
        response.on("end", () => {
          try {
            resolve(JSON.parse(raw));
          } catch (error) {
            reject(error);
          }
        });
      })
      .on("error", reject);
  });
}

async function waitForDebugger(port, timeoutMs) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    try {
      const targets = await httpJson(`http://127.0.0.1:${port}/json/list`);
      const pageTarget =
        targets.find((target) => target.type === "page") ?? targets[0];
      if (pageTarget?.webSocketDebuggerUrl) {
        return pageTarget.webSocketDebuggerUrl;
      }
    } catch {}
    await sleep(200);
  }

  throw new Error(`Timed out waiting for Chrome debugger on port ${port}.`);
}

function createCdpClient(wsUrl) {
  const socket = new WebSocket(wsUrl);
  let nextId = 1;
  const pending = new Map();

  const ready = new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });

  socket.addEventListener("message", (event) => {
    const payload = JSON.parse(String(event.data));
    if (!("id" in payload)) return;

    const pendingRequest = pending.get(payload.id);
    if (!pendingRequest) return;

    pending.delete(payload.id);
    if (payload.error) {
      pendingRequest.reject(
        new Error(payload.error.message ?? "Unknown CDP error")
      );
      return;
    }

    pendingRequest.resolve(payload.result);
  });

  return {
    async send(method, params = {}) {
      await ready;
      const id = nextId;
      nextId += 1;

      return new Promise((resolve, reject) => {
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      });
    },
    close() {
      socket.close();
    },
  };
}

async function waitForPageContent(client, options) {
  const startedAt = Date.now();
  while (Date.now() - startedAt < options.timeoutMs) {
    const evaluation = await client.send("Runtime.evaluate", {
      expression: `(() => {
        const text = document.body ? document.body.innerText : "";
        const style = document.body ? getComputedStyle(document.body) : null;
        const cssHrefReady = Array.from(document.querySelectorAll('link[rel="stylesheet"]'))
          .some((link) => link.href.includes("/assets/main-"));
        const bodyStyled = !!style && style.backgroundImage && style.backgroundImage !== "none";
        return {
          text,
          cssHrefReady,
          bodyStyled,
          readyState: document.readyState,
        };
      })()`,
      returnByValue: true,
    });
    const state = evaluation.result?.value ?? {};
    const text = String(state.text ?? "");

    const hasAllRequired = options.waitTexts.every((entry) => text.includes(entry));
    const hasForbidden = options.forbidTexts.some((entry) => text.includes(entry));
    const cssReady =
      state.readyState === "complete" &&
      state.cssHrefReady === true &&
      state.bodyStyled === true;

    if (hasAllRequired && !hasForbidden && cssReady) {
      return;
    }

    await sleep(250);
  }

  throw new Error(
    `Timed out waiting for hydrated content. Required: ${options.waitTexts.join(", ")}`
  );
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const chrome = spawn(
    options.chromePath,
    [
      "--headless=new",
      "--disable-gpu",
      "--no-sandbox",
      `--remote-debugging-port=${options.debugPort}`,
      `--window-size=${options.viewport}`,
      "about:blank",
    ],
    { stdio: ["ignore", "ignore", "pipe"] }
  );

  let chromeStderr = "";
  chrome.stderr.on("data", (chunk) => {
    chromeStderr += String(chunk);
  });

  try {
    const wsUrl = await waitForDebugger(options.debugPort, options.timeoutMs);
    const client = createCdpClient(wsUrl);

    await client.send("Page.enable");
    await client.send("Runtime.enable");
    await client.send("Page.navigate", { url: options.url });
    await waitForPageContent(client, options);

    const html = await client.send("Runtime.evaluate", {
      expression: "document.documentElement.outerHTML",
      returnByValue: true,
    });
    const screenshot = await client.send("Page.captureScreenshot", {
      format: "png",
      captureBeyondViewport: options.fullPage,
      fromSurface: true,
    });

    await fs.writeFile(options.screenshot, Buffer.from(screenshot.data, "base64"));
    if (options.dom) {
      await fs.writeFile(options.dom, String(html.result?.value ?? ""), "utf8");
    }

    client.close();
  } catch (error) {
    chrome.kill("SIGKILL");
    throw new Error(`${error.message}\nChrome stderr:\n${chromeStderr}`);
  }

  chrome.kill("SIGKILL");
  process.stdout.write(`Verified and captured ${options.url}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error.message}\n`);
  process.exit(1);
});
