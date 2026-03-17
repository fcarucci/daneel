#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0

import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { chromium } from "playwright";

function parseArgs(argv) {
  const options = {
    url: "http://127.0.0.1:4127/",
    screenshot: "/tmp/daneel-live.png",
    dom: "/tmp/daneel-live.html",
    video: "",
    timeoutMs: 40_000,
    viewport: "1600,1400",
    browserPath: "",
    title: "Daneel",
    waitTexts: [],
    forbidTexts: [],
    minLatestSessionCount: 0,
    fullPage: true,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];

    if (arg === "--url") {
      options.url = next;
      index += 1;
    } else if (arg === "--screenshot") {
      options.screenshot = next;
      index += 1;
    } else if (arg === "--dom") {
      options.dom = next;
      index += 1;
    } else if (arg === "--video") {
      options.video = next;
      index += 1;
    } else if (arg === "--timeout-ms") {
      options.timeoutMs = Number.parseInt(next, 10);
      index += 1;
    } else if (arg === "--viewport") {
      options.viewport = next;
      index += 1;
    } else if (arg === "--browser-path") {
      options.browserPath = next;
      index += 1;
    } else if (arg === "--title") {
      options.title = next;
      index += 1;
    } else if (arg === "--wait-text") {
      options.waitTexts.push(next);
      index += 1;
    } else if (arg === "--forbid-text") {
      options.forbidTexts.push(next);
      index += 1;
    } else if (arg === "--min-latest-session-count") {
      options.minLatestSessionCount = Number.parseInt(next, 10);
      index += 1;
    } else if (arg === "--no-full-page") {
      options.fullPage = false;
    }
  }

  return options;
}

function parseCommand(argv) {
  const [maybeCommand, ...rest] = argv;
  if (maybeCommand === "verify" || maybeCommand === "upload") {
    return { command: maybeCommand, argv: rest };
  }

  return { command: "verify", argv };
}

function parseViewport(value) {
  const [width, height] = value
    .split(",")
    .map((entry) => Number.parseInt(entry, 10));
  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    throw new Error(`Invalid --viewport value: ${value}`);
  }

  return { width, height };
}

async function ensureParentDir(filePath) {
  await fs.mkdir(path.dirname(filePath), { recursive: true });
}

function replaceExtension(filePath, extension) {
  const currentExtension = path.extname(filePath);
  if (!currentExtension) {
    return `${filePath}${extension}`;
  }

  return `${filePath.slice(0, -currentExtension.length)}${extension}`;
}

function resolveVideoOutputPath(requestedVideoPath) {
  if (!requestedVideoPath) {
    return "";
  }

  if (path.isAbsolute(requestedVideoPath)) {
    return requestedVideoPath;
  }

  if (path.dirname(requestedVideoPath) !== ".") {
    return requestedVideoPath;
  }

  return path.join("videos", requestedVideoPath);
}

function routeSlugFromUrl(url) {
  const pathname = new URL(url).pathname.replace(/^\/+|\/+$/g, "");
  return pathname ? pathname.replace(/[^a-zA-Z0-9]+/g, "-") : "home";
}

function timestampStamp(date = new Date()) {
  const pad = (value) => String(value).padStart(2, "0");
  return (
    `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_` +
    `${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`
  );
}

function uniquifyPathWithRouteAndTime(filePath, url) {
  const extension = path.extname(filePath);
  const directory = path.dirname(filePath);
  const uniqueName = `${routeSlugFromUrl(url)}_${timestampStamp()}${extension}`;
  return path.join(directory, uniqueName);
}

async function convertVideoToMp4(inputPath, outputPath) {
  await ensureParentDir(outputPath);

  await new Promise((resolve, reject) => {
    const ffmpeg = spawn("ffmpeg", [
      "-y",
      "-i",
      inputPath,
      "-c:v",
      "libx264",
      "-pix_fmt",
      "yuv420p",
      "-movflags",
      "+faststart",
      outputPath,
    ]);

    let stderr = "";
    ffmpeg.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    ffmpeg.on("error", (error) => {
      reject(
        new Error(
          `Failed to start ffmpeg while converting ${inputPath} to ${outputPath}: ${error.message}`
        )
      );
    });
    ffmpeg.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(
        new Error(
          `ffmpeg exited with code ${code} while converting ${inputPath} to ${outputPath}\n${stderr}`
        )
      );
    });
  });
}

async function persistRecordedVideo(recordedVideoPath, requestedVideoPath, url) {
  if (!requestedVideoPath) {
    return null;
  }

  const resolvedVideoPath = uniquifyPathWithRouteAndTime(
    resolveVideoOutputPath(requestedVideoPath),
    url
  );
  await ensureParentDir(resolvedVideoPath);

  const requestedExtension = path.extname(resolvedVideoPath).toLowerCase();
  if (requestedExtension === ".mp4") {
    const webmPath = replaceExtension(resolvedVideoPath, ".webm");
    await fs.copyFile(recordedVideoPath, webmPath);
    try {
      await convertVideoToMp4(webmPath, resolvedVideoPath);
    } finally {
      await fs.rm(webmPath, { force: true });
    }
    return resolvedVideoPath;
  }

  await fs.copyFile(recordedVideoPath, resolvedVideoPath);
  return resolvedVideoPath;
}

function parseUploadArgs(argv) {
  const options = {
    video: "",
    tag: "verification-artifacts",
    releaseName: "Verification Artifacts",
    releaseBody: "Temporary verification media uploaded from local review runs.",
    label: "",
    pr: 0,
    route: "/agents",
    latestSessionCount: "",
    connectedRibbon: "",
    screenshot: "",
    dom: "",
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];

    if (arg === "--video") {
      options.video = next;
      index += 1;
    } else if (arg === "--tag") {
      options.tag = next;
      index += 1;
    } else if (arg === "--release-name") {
      options.releaseName = next;
      index += 1;
    } else if (arg === "--release-body") {
      options.releaseBody = next;
      index += 1;
    } else if (arg === "--label") {
      options.label = next;
      index += 1;
    } else if (arg === "--pr") {
      options.pr = Number.parseInt(next, 10);
      index += 1;
    } else if (arg === "--route") {
      options.route = next;
      index += 1;
    } else if (arg === "--latest-session-count") {
      options.latestSessionCount = next;
      index += 1;
    } else if (arg === "--connected-ribbon") {
      options.connectedRibbon = next;
      index += 1;
    } else if (arg === "--screenshot") {
      options.screenshot = next;
      index += 1;
    } else if (arg === "--dom") {
      options.dom = next;
      index += 1;
    }
  }

  return options;
}

async function runCommand(command, args) {
  const child = spawn(command, args, {
    cwd: process.cwd(),
    stdio: ["ignore", "pipe", "pipe"],
  });

  let stdout = "";
  let stderr = "";

  child.stdout.on("data", (chunk) => {
    stdout += chunk.toString();
  });
  child.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });

  await new Promise((resolve, reject) => {
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(
        new Error(
          `${command} ${args.join(" ")} failed with code ${code}\n${stderr || stdout}`
        )
      );
    });
  });

  return stdout.trim();
}

async function runGitHubAdmin(args) {
  const stdout = await runCommand("node", ["scripts/github-admin.mjs", ...args]);
  return stdout ? JSON.parse(stdout) : null;
}

async function waitForLiveRoute(page, options) {
  await page.waitForFunction(
    ({ waitTexts, forbidTexts, minLatestSessionCount }) => {
      const body = document.body;
      if (!body) return false;

      const text = body.innerText ?? "";
      const lowerText = text.toLowerCase();
      const stylesheetReady = Array.from(
        document.querySelectorAll('link[rel="stylesheet"]')
      ).some((link) => link.href.includes("/assets/main-"));
      const backgroundImage = getComputedStyle(body).backgroundImage;
      const bodyStyled = backgroundImage && backgroundImage !== "none";
      const hasAllRequired = waitTexts.every((entry) =>
        lowerText.includes(entry.toLowerCase())
      );
      const hasForbidden = forbidTexts.some((entry) =>
        lowerText.includes(entry.toLowerCase())
      );
      const latestSessionCount = Array.from(document.querySelectorAll("p"))
        .map((node) => node.textContent?.trim() ?? "")
        .filter((content) => content.startsWith("Latest session:")).length;

      return (
        document.readyState === "complete" &&
        stylesheetReady &&
        bodyStyled &&
        hasAllRequired &&
        !hasForbidden &&
        latestSessionCount >= minLatestSessionCount
      );
    },
    {
      waitTexts: options.waitTexts,
      forbidTexts: options.forbidTexts,
      minLatestSessionCount: options.minLatestSessionCount,
    },
    { timeout: options.timeoutMs }
  );
}

async function captureSummary(page) {
  return page.evaluate(() => {
    const latestSessionCount = Array.from(document.querySelectorAll("p"))
      .map((node) => node.textContent?.trim() ?? "")
      .filter((content) => content.startsWith("Latest session:")).length;
    const bodyText = document.body?.innerText ?? "";

    return {
      title: document.title,
      connectedRibbonPresent:
        bodyText.includes("Connected") || bodyText.includes("Healthy"),
      latestSessionCount,
    };
  });
}

async function verifyRoute(argv) {
  const options = parseArgs(argv);
  const browser = await chromium.launch({
    executablePath: options.browserPath || undefined,
    headless: true,
  });

  try {
    const viewport = parseViewport(options.viewport);
    const context = await browser.newContext({
      viewport,
      recordVideo: options.video
        ? {
            dir: process.env.TMPDIR || "/tmp",
            size: viewport,
          }
        : undefined,
    });
    const page = await context.newPage();
    await page.goto(options.url, { waitUntil: "domcontentloaded" });
    await waitForLiveRoute(page, options);

    const html = await page.content();
    const summary = await captureSummary(page);

    assert.equal(summary.title, options.title, `expected the page title to be ${options.title}`);
    assert.ok(
      summary.latestSessionCount >= options.minLatestSessionCount,
      `expected at least ${options.minLatestSessionCount} latest-session entries`
    );

    await ensureParentDir(options.screenshot);
    await ensureParentDir(options.dom);
    await page.screenshot({
      path: options.screenshot,
      fullPage: options.fullPage,
    });
    await fs.writeFile(options.dom, html, "utf8");
    const recordedVideoPath = options.video ? await page.video()?.path() : null;
    await context.close();

    const persistedVideoPath =
      options.video && recordedVideoPath
        ? await persistRecordedVideo(recordedVideoPath, options.video, options.url)
        : null;

    process.stdout.write(
      JSON.stringify(
        {
          verified: true,
          url: options.url,
          screenshot: options.screenshot,
          dom: options.dom,
          video: persistedVideoPath,
          title: summary.title,
          latestSessionCount: summary.latestSessionCount,
          connectedRibbonPresent: summary.connectedRibbonPresent,
        },
        null,
        2
      ) + "\n"
    );
  } finally {
    await browser.close();
  }
}

async function uploadVideo(argv) {
  const options = parseUploadArgs(argv);
  if (!options.video) {
    throw new Error("upload requires --video <path>.");
  }

  const resolvedVideoPath = path.resolve(options.video);
  await fs.access(resolvedVideoPath);

  await runGitHubAdmin([
    "ensure-release",
    "--tag",
    options.tag,
    "--name",
    options.releaseName,
    "--body",
    options.releaseBody,
    "--draft",
    "--prerelease",
  ]);

  const uploadResult = await runGitHubAdmin([
    "upload-release-asset",
    "--tag",
    options.tag,
    "--file",
    resolvedVideoPath,
    ...(options.label ? ["--label", options.label] : []),
  ]);

  let prComment = null;
  if (Number.isInteger(options.pr) && options.pr > 0) {
    prComment = await runGitHubAdmin([
      "comment-pr-verification",
      "--number",
      String(options.pr),
      "--route",
      options.route,
      "--artifact-url",
      uploadResult.asset.download_url,
      ...(options.latestSessionCount
        ? ["--latest-session-count", String(options.latestSessionCount)]
        : []),
      ...(options.connectedRibbon
        ? ["--connected-ribbon", String(options.connectedRibbon)]
        : []),
      ...(options.screenshot ? ["--screenshot", options.screenshot] : []),
      ...(options.dom ? ["--dom", options.dom] : []),
      "--video",
      resolvedVideoPath,
    ]);
  }

  process.stdout.write(
    JSON.stringify(
      {
        uploaded: true,
        video: resolvedVideoPath,
        release: uploadResult.release,
        asset: uploadResult.asset,
        prComment,
      },
      null,
      2
    ) + "\n"
  );
}

async function main() {
  const { command, argv } = parseCommand(process.argv.slice(2));
  if (command === "upload") {
    await uploadVideo(argv);
    return;
  }

  await verifyRoute(argv);
}

main().catch((error) => {
  process.stderr.write(`${error.stack ?? error.message}\n`);
  process.exit(1);
});
