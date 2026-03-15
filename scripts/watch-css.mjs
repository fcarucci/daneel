#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0


import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const ROOT = process.cwd();
const WATCH_ROOTS = ["src", "styles"];
const watchers = [];

let buildRunning = false;
let rebuildQueued = false;
let debounceTimer = null;

function timestamp() {
  return new Date().toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function log(message) {
  process.stdout.write(`[css] ${message}\n`);
}

function listDirectories(root) {
  const directories = [root];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    if (!entry.isDirectory()) {
      continue;
    }

    if (entry.name === "target" || entry.name === "node_modules" || entry.name.startsWith(".")) {
      continue;
    }

    directories.push(...listDirectories(path.join(root, entry.name)));
  }

  return directories;
}

function runBuild() {
  if (buildRunning) {
    rebuildQueued = true;
    return;
  }

  buildRunning = true;
  const startedAt = Date.now();
  const child = spawn(
    "npx",
    ["@tailwindcss/cli", "-i", "./styles/app.css", "-o", "./assets/main.css", "--minify"],
    {
      cwd: ROOT,
      stdio: "pipe",
    }
  );

  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += String(chunk);
  });

  child.on("exit", (code) => {
    buildRunning = false;
    if (code === 0) {
      const duration = Date.now() - startedAt;
      log(`rebuilt in ${duration}ms at ${timestamp()}`);
    } else {
      log(`build failed\n${stderr}`.trim());
    }

    if (rebuildQueued) {
      rebuildQueued = false;
      runBuild();
    }
  });
}

function scheduleBuild() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    runBuild();
  }, 120);
}

function startWatching() {
  for (const relativeRoot of WATCH_ROOTS) {
    const absoluteRoot = path.join(ROOT, relativeRoot);
    if (!fs.existsSync(absoluteRoot)) {
      continue;
    }

    for (const directory of listDirectories(absoluteRoot)) {
      const watcher = fs.watch(directory, () => {
        scheduleBuild();
      });
      watchers.push(watcher);
    }
  }
}

function cleanupAndExit(code) {
  clearTimeout(debounceTimer);
  for (const watcher of watchers) {
    watcher.close();
  }
  process.exit(code);
}

process.on("SIGINT", () => cleanupAndExit(0));
process.on("SIGTERM", () => cleanupAndExit(0));

log("watching src/ and styles/ for Tailwind rebuilds");
startWatching();
