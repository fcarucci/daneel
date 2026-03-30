// SPDX-License-Identifier: Apache-2.0

import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

const DEFAULT_REPO = "fcarucci/daneel";
const PROJECT_NUMBER = 1;

const LABEL_DEFS = {
  "poc-v1": { color: "0e8a16", description: "Proof-of-concept V1 tracking work" },
  "priority/p0": { color: "b60205", description: "Highest priority work" },
  "priority/p1": { color: "d93f0b", description: "Important next work" },
  "priority/p2": { color: "fbca04", description: "Useful later work" },
  frontend: { color: "1d76db", description: "Frontend or UI work" },
  backend: { color: "5319e7", description: "Backend or server-side work" },
  gateway: { color: "0052cc", description: "OpenClaw gateway integration" },
  adapter: { color: "006b75", description: "Adapter contract or implementation work" },
  graph: { color: "0e8a16", description: "Graph modeling or rendering work" },
  testing: { color: "bfd4f2", description: "Testing and verification work" },
  design: { color: "c5def5", description: "Product, UX, or technical design work" },
  blocked: { color: "000000", description: "Blocked by an external dependency or decision" },
  skipped: { color: "6e7781", description: "Intentionally skipped from the current plan" },
  "not-to-implement": { color: "8250df", description: "Explicitly decided not to implement" },
  task: { color: "ededed", description: "Tracked implementation task" },
  bug: { color: "d73a4a", description: "Something is not working" },
  enhancement: { color: "a2eeef", description: "New feature or improvement" },
};


function usage() {
  console.log(`Usage:
  node skills/github-admin/scripts/github-admin.mjs <command> [options]

Commands:
  sync-labels
  list-issues [--state <open|closed|all>] [--title-prefix <text>] [--title-contains <text>] [--limit <n>]
  list-tasks [--limit <n>]
  issue-comment --action <list|delete> (list: --issue <n>; delete: --comment-id <n>)
  update-issue --number <n> [--title <title>] [--body <text>] [--body-file <path>] [--state <open|closed>] [--labels <a,b,c>] [--assignees <login,login>] [--milestone <n>]
  create-issue --title <title> [--body <text>] [--body-file <path>] [--labels <a,b,c>] [--milestone <n>] [--assignees <login,login>]
  create-project --title <title> [--private] [--dry-run]   (ProjectV2 owned by GITHUB_REPOSITORY owner; public by default)
  get-issue --number <n>
  delete-issue --number <n>
  label-issue --action <add|remove> (add: --number <n> --labels <a,b>; remove: --number <n> --label <name>)
  list-prs [--state <open|closed|all>]   (default: open)
  comment-issue --number <n> --body <text>
  comment-pr --number <n> --body <text>   (alias: same as comment-issue; works for PRs)
  ensure-release --tag <tag> [--name <title>] [--body <text>] [--draft] [--prerelease]
  upload-release-asset --tag <tag> --file <path> [--label <text>]
  comment-pr-verification --number <n> --artifact-url <url> [--route <route>] [--latest-session-count <n>] [--connected-ribbon <true|false>] [--screenshot <path>] [--dom <path>]
  release-asset --action <list|delete> (list: --tag <tag>; delete: --asset-id <n>)
  pr-review --action <list|resolve> (list: --number <n>; resolve: --thread-id <id>)
  merge-pr --number <n> [--method <merge|squash|rebase>] [--title <title>] [--message <text>]
  create-pr --head <branch> --title <title> [--base <branch>] [--body <text>] [--issue <n>] [--draft]
  update-pr --number <n> [--title <title>] [--body <text>] [--base <branch>] [--state <open|closed>]
  link-pr-task --pr <n> --issue <n> [--close]
  project --action <link-prs|close-project> [--title-prefix <text>] [--dry-run]
  set-project-visibility --public|--private [--number <n>] [--dry-run]   (user ProjectV2; default number from GITHUB_PROJECT_NUMBER)
  set-issue-status --issues <n,n,...> --status <status>
  report

Environment:
  GITHUB_TOKEN or GITHUB_PERSONAL_ACCESS_TOKEN
  GITHUB_REPOSITORY (default: ${DEFAULT_REPO})
  GITHUB_PROJECT_NUMBER (default: ${PROJECT_NUMBER})
`);
}

function token() {
  const value =
    process.env.GITHUB_TOKEN ||
    process.env.GITHUB_PERSONAL_ACCESS_TOKEN ||
    envFileToken();
  if (!value) {
    throw new Error(
      "Set GITHUB_TOKEN or GITHUB_PERSONAL_ACCESS_TOKEN before running this script.",
    );
  }
  return value;
}

let cachedEnvFileToken;

function envFileToken() {
  if (cachedEnvFileToken !== undefined) {
    return cachedEnvFileToken;
  }

  try {
    const envPath = path.join(os.homedir(), ".env");
    cachedEnvFileToken = extractTokenFromEnvFile(envPath);
  } catch {
    cachedEnvFileToken = null;
  }

  return cachedEnvFileToken;
}

function extractTokenFromEnvFile(envPath) {
  if (!existsSync(envPath)) {
    return null;
  }

  const raw = readFileSync(envPath, "utf8");
  for (const line of raw.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) {
      continue;
    }

    const match = trimmed.match(
      /^export\s+GITHUB_PERSONAL_ACCESS_TOKEN\s*=\s*(?:"([^"]*)"|'([^']*)'|(.+))$/,
    );
    if (match) {
      return (match[1] || match[2] || match[3] || "").trim();
    }
  }

  return null;
}

function repoParts() {
  const repo = process.env.GITHUB_REPOSITORY || DEFAULT_REPO;
  const [owner, name] = repo.split("/");
  if (!owner || !name) {
    throw new Error(`Invalid GITHUB_REPOSITORY value: ${repo}`);
  }
  return { owner, repo: name };
}

function projectNumber() {
  return Number(process.env.GITHUB_PROJECT_NUMBER || PROJECT_NUMBER);
}

function parseArgs(argv) {
  const [command, ...rest] = argv;
  const options = {};
  for (let i = 0; i < rest.length; i += 1) {
    const arg = rest[i];
    if (!arg.startsWith("--")) {
      continue;
    }
    const key = arg.slice(2);
    const next = rest[i + 1];
    if (!next || next.startsWith("--")) {
      options[key] = true;
    } else {
      options[key] = next;
      i += 1;
    }
  }
  return { command, options };
}

function parseCsvList(value) {
  return String(value || "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);
}

async function rest(method, path, body) {
  const response = await fetch(`https://api.github.com${path}`, {
    method,
    headers: {
      Authorization: `token ${token()}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      "User-Agent": "daneel-github-admin",
      "Content-Type": "application/json",
    },
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!response.ok) {
    throw new Error(`${method} ${path} failed: ${response.status} ${await response.text()}`);
  }

  const text = await response.text();
  return text ? JSON.parse(text) : null;
}

async function graphql(query, variables = {}) {
  const response = await fetch("https://api.github.com/graphql", {
    method: "POST",
    headers: {
      Authorization: `token ${token()}`,
      Accept: "application/vnd.github+json",
      "User-Agent": "daneel-github-admin",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ query, variables }),
  });

  if (!response.ok) {
    throw new Error(`GraphQL failed: ${response.status} ${await response.text()}`);
  }

  const payload = await response.json();
  if (payload.errors?.length) {
    throw new Error(JSON.stringify(payload.errors));
  }
  return payload.data;
}

async function allIssues(state = "all") {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/issues?state=${state}&per_page=100`);
}

async function allPulls(state = "open") {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/pulls?state=${state}&per_page=100`);
}

async function allPullRequests() {
  const { owner, repo } = repoParts();
  const out = [];
  for (let page = 1; page <= 10; page += 1) {
    const batch = await rest(
      "GET",
      `/repos/${owner}/${repo}/pulls?state=all&per_page=100&page=${page}`,
    );
    if (!batch.length) {
      break;
    }
    out.push(...batch);
    if (batch.length < 100) {
      break;
    }
  }
  return out;
}

function taskIdFromIssueTitle(title) {
  const match = title.match(/\b(T\d+\.\d+)\b/);
  return match ? match[1] : null;
}

function taskIdFromPullTitle(title) {
  const match = title.match(/\b(T\d+\.\d+)\b/);
  return match ? match[1] : null;
}

function extractClosingIssueRefsFromText(text) {
  if (!text) {
    return [];
  }
  const refs = new Set();
  const re = /(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s*:?\s*#(\d+)/gi;
  let match;
  while ((match = re.exec(text)) !== null) {
    refs.add(Number(match[1]));
  }
  return [...refs];
}

function buildIssueToPullMap(issues, pulls) {
  const issueByNum = new Map(issues.map((issue) => [issue.number, issue]));
  const byIssue = new Map();

  function addCandidate(issueNumber, pullNumber, score, reason) {
    let inner = byIssue.get(issueNumber);
    if (!inner) {
      inner = new Map();
      byIssue.set(issueNumber, inner);
    }
    const prev = inner.get(pullNumber);
    if (!prev || score > prev.score) {
      inner.set(pullNumber, { pr: pullNumber, score, reason });
    }
  }

  for (const pr of pulls) {
    const combined = `${pr.title || ""}\n${pr.body || ""}`;
    for (const num of extractClosingIssueRefsFromText(combined)) {
      if (!issueByNum.has(num)) {
        continue;
      }
      addCandidate(num, pr.number, 10, "closing-keyword");
    }

    const prTask = taskIdFromPullTitle(pr.title || "");
    if (prTask) {
      for (const iss of issues) {
        const issueTask = taskIdFromIssueTitle(iss.title);
        if (issueTask === prTask) {
          addCandidate(iss.number, pr.number, 8, "task-id");
        }
      }
    }
  }

  const result = new Map();
  for (const iss of issues) {
    const inner = byIssue.get(iss.number);
    if (!inner?.size) {
      continue;
    }
    const candidates = [...inner.values()];
    candidates.sort((a, b) => {
      if (b.score !== a.score) {
        return b.score - a.score;
      }
      return b.pr - a.pr;
    });
    result.set(iss.number, candidates[0]);
  }
  return result;
}

function pullBodyClosesIssue(body, issueNumber) {
  return new RegExp(`(?:close|fix|resolve)[sd]?\\s*:?\\s*#${issueNumber}\\b`, "i").test(
    body || "",
  );
}

function issueHasLinkedPullComment(comments, pullNumber) {
  const re = new RegExp(`Linked pull request:\\s*\\[#${pullNumber}\\]`, "i");
  return comments.some((comment) => re.test(comment.body || ""));
}

async function ensureIssueLinkedToPull(issueNumber, pullNumber, dryRun) {
  const { owner, repo } = repoParts();
  const pull = await getPullRequest(pullNumber);
  const comments = await rest(
    "GET",
    `/repos/${owner}/${repo}/issues/${issueNumber}/comments?per_page=100`,
  );

  let patchedBody = false;
  let postedComment = false;

  if (!pullBodyClosesIssue(pull.body, issueNumber)) {
    patchedBody = true;
    if (!dryRun) {
      const nextBody = `${pull.body || ""}\n\nCloses #${issueNumber}`.trim();
      await rest("PATCH", `/repos/${owner}/${repo}/pulls/${pullNumber}`, {
        body: nextBody,
      });
    }
  }

  if (!issueHasLinkedPullComment(comments, pullNumber)) {
    postedComment = true;
    if (!dryRun) {
      await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
        body: `Linked pull request: [#${pullNumber}](${pullUrl(pullNumber)})`,
      });
    }
  }

  return { patchedBody, postedComment };
}

async function linkPrs(options) {
  const dryRun = Boolean(options["dry-run"]);
  const titlePrefix = options["title-prefix"] || null;
  let issues = (await allIssues()).filter((issue) => !issue.pull_request);
  if (titlePrefix) {
    issues = issues.filter((issue) => issue.title.startsWith(titlePrefix));
  }
  const pulls = await allPullRequests();
  const map = buildIssueToPullMap(issues, pulls);

  const linked = [];
  const alreadyLinked = [];
  const unmapped = [];

  for (const issue of issues.sort((a, b) => a.number - b.number)) {
    const pick = map.get(issue.number);
    if (!pick) {
      unmapped.push({
        number: issue.number,
        title: issue.title,
      });
      continue;
    }

    const actions = await ensureIssueLinkedToPull(issue.number, pick.pr, dryRun);
    if (!actions.patchedBody && !actions.postedComment) {
      alreadyLinked.push({
        issue: issue.number,
        pull: pick.pr,
        reason: pick.reason,
      });
      continue;
    }

    linked.push({
      issue: issue.number,
      pull: pick.pr,
      reason: pick.reason,
      patchedPullBody: actions.patchedBody,
      postedIssueComment: actions.postedComment,
      dryRun,
    });
  }

  console.log(
    JSON.stringify(
      {
        dryRun,
        mappedIssueCount: map.size,
        issueCount: issues.length,
        linked,
        alreadyLinked,
        unmapped,
      },
      null,
      2,
    ),
  );
}

async function closeProject(options) {
  const dryRun = Boolean(options["dry-run"]);
  const data = await projectData();
  const project = data.user.projectV2;
  if (!project) {
    throw new Error(`No project found for user login and number ${projectNumber()}.`);
  }

  if (project.closed) {
    console.log(
      JSON.stringify(
        {
          message: "Project is already closed.",
          number: projectNumber(),
          title: project.title,
          closed: project.closed,
        },
        null,
        2,
      ),
    );
    return;
  }

  if (dryRun) {
    console.log(
      JSON.stringify(
        {
          dryRun: true,
          number: projectNumber(),
          title: project.title,
          wouldSetClosed: true,
        },
        null,
        2,
      ),
    );
    return;
  }

  const updated = await graphql(
    `
    mutation($projectId: ID!) {
      updateProjectV2(input: { projectId: $projectId, closed: true }) {
        projectV2 {
          id
          title
          closed
        }
      }
    }
    `,
    { projectId: project.id },
  );

  console.log(
    JSON.stringify(
      {
        message: "Project closed.",
        number: projectNumber(),
        project: updated.updateProjectV2.projectV2,
      },
      null,
      2,
    ),
  );
}

async function allReleases() {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/releases?per_page=100`);
}

async function getReleaseByTag(tag) {
  const { owner, repo } = repoParts();
  try {
    return await rest("GET", `/repos/${owner}/${repo}/releases/tags/${encodeURIComponent(tag)}`);
  } catch (error) {
    if (String(error.message).includes("404")) {
      const releases = await allReleases();
      return releases.find((release) => release.tag_name === tag) || null;
    }
    throw error;
  }
}

async function createRelease({ tag, name, body, draft, prerelease }) {
  const { owner, repo } = repoParts();
  return rest("POST", `/repos/${owner}/${repo}/releases`, {
    tag_name: tag,
    name: name || tag,
    body: body || "",
    draft: Boolean(draft),
    prerelease: Boolean(prerelease),
  });
}

async function deleteReleaseAsset(assetId) {
  const { owner, repo } = repoParts();
  return rest("DELETE", `/repos/${owner}/${repo}/releases/assets/${assetId}`);
}

function contentTypeForAsset(filePath) {
  const extension = path.extname(filePath).toLowerCase();
  switch (extension) {
    case ".mp4":
      return "video/mp4";
    case ".webm":
      return "video/webm";
    case ".png":
      return "image/png";
    case ".html":
      return "text/html";
    case ".json":
      return "application/json";
    default:
      return "application/octet-stream";
  }
}

async function uploadReleaseAssetBinary(uploadUrl, filePath, label) {
  const fileName = path.basename(filePath);
  const fileBuffer = await readFile(filePath);
  const target = new URL(uploadUrl.replace(/\{\?name,label\}$/, ""));
  target.searchParams.set("name", fileName);
  if (label) {
    target.searchParams.set("label", label);
  }

  const response = await fetch(target, {
    method: "POST",
    headers: {
      Authorization: `token ${token()}`,
      Accept: "application/vnd.github+json",
      "User-Agent": "daneel-github-admin",
      "Content-Type": contentTypeForAsset(filePath),
      "Content-Length": String(fileBuffer.byteLength),
    },
    body: fileBuffer,
  });

  if (!response.ok) {
    throw new Error(
      `Asset upload failed: ${response.status} ${await response.text()}`
    );
  }

  return response.json();
}

async function getIssue(issueNumber) {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/issues/${issueNumber}`);
}

async function getPullRequest(pullNumber) {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/pulls/${pullNumber}`);
}

async function allMilestones() {
  const { owner, repo } = repoParts();
  return rest("GET", `/repos/${owner}/${repo}/milestones?state=all&per_page=100`);
}


async function syncLabels() {
  const { owner, repo } = repoParts();
  const existing = await rest("GET", `/repos/${owner}/${repo}/labels?per_page=100`);
  const byName = new Map(existing.map((label) => [label.name, label]));

  for (const [name, meta] of Object.entries(LABEL_DEFS)) {
    if (byName.has(name)) {
      await rest(
        "PATCH",
        `/repos/${owner}/${repo}/labels/${encodeURIComponent(name)}`,
        { new_name: name, ...meta },
      );
    } else {
      await rest("POST", `/repos/${owner}/${repo}/labels`, { name, ...meta });
    }
  }

  console.log(`Synced ${Object.keys(LABEL_DEFS).length} labels.`);
}


async function addIssueToProject(projectId, contentId) {
  const data = await graphql(
    `
    mutation($project:ID!, $content:ID!) {
      addProjectV2ItemById(input:{projectId:$project, contentId:$content}) {
        item {
          id
        }
      }
    }
    `,
    {
      project: projectId,
      content: contentId,
    },
  );

  return data.addProjectV2ItemById.item.id;
}

function resolveUserProjectNumberArg(raw) {
  if (raw == null || raw === "") {
    return projectNumber();
  }
  const n = Number(raw);
  if (!Number.isInteger(n) || n < 1) {
    throw new Error("set-project-visibility --number must be a positive integer.");
  }
  return n;
}

async function fetchUserProjectV2Summary(overrideNumber) {
  const { owner } = repoParts();
  const number = resolveUserProjectNumberArg(overrideNumber);
  const data = await graphql(
    `
    query($owner:String!, $number:Int!) {
      user(login:$owner) {
        projectV2(number:$number) {
          id
          title
          public
          url
          closed
        }
      }
    }
    `,
    { owner, number },
  );
  return { project: data.user.projectV2, number };
}

async function setProjectVisibility(options) {
  const dryRun = Boolean(options["dry-run"]);
  const wantPublic = Boolean(options.public);
  const wantPrivate = Boolean(options.private);
  if (wantPublic === wantPrivate) {
    throw new Error(
      "set-project-visibility requires exactly one of --public or --private.",
    );
  }
  const { project, number } = await fetchUserProjectV2Summary(options.number);
  if (!project) {
    throw new Error(
      `No user ProjectV2 found for login and number ${number}. Check GITHUB_REPOSITORY owner and --number.`,
    );
  }
  if (dryRun) {
    console.log(
      JSON.stringify(
        {
          dryRun: true,
          wouldSetPublic: wantPublic,
          project: {
            id: project.id,
            title: project.title,
            public: project.public,
            url: project.url,
            closed: project.closed,
          },
        },
        null,
        2,
      ),
    );
    return;
  }
  const updated = await graphql(
    `
    mutation($projectId: ID!, $public: Boolean!) {
      updateProjectV2(input: { projectId: $projectId, public: $public }) {
        projectV2 {
          id
          title
          public
          url
          closed
        }
      }
    }
    `,
    { projectId: project.id, public: wantPublic },
  );
  console.log(
    JSON.stringify({ project: updated.updateProjectV2.projectV2 }, null, 2),
  );
}

async function fetchRepositoryOwnerNodeId() {
  const { owner, repo } = repoParts();
  const data = await graphql(
    `
    query($owner: String!, $name: String!) {
      repository(owner: $owner, name: $name) {
        owner {
          __typename
          id
        }
      }
    }
    `,
    { owner, name: repo },
  );
  const ownerNode = data.repository?.owner;
  if (!ownerNode?.id) {
    throw new Error(
      `Could not resolve owner id for repository ${owner}/${repo}.`,
    );
  }
  return { ownerId: ownerNode.id, ownerTypename: ownerNode.__typename };
}

async function createProjectCommand(options) {
  const dryRun = Boolean(options["dry-run"]);
  const title = String(options.title || "").trim();
  if (!title) {
    throw new Error("create-project requires --title <text>.");
  }
  const stayPrivate = Boolean(options.private);
  const { ownerId, ownerTypename } = await fetchRepositoryOwnerNodeId();
  if (dryRun) {
    console.log(
      JSON.stringify(
        {
          dryRun: true,
          title,
          ownerTypename,
          wouldCreatePublic: !stayPrivate,
        },
        null,
        2,
      ),
    );
    return;
  }
  const created = await graphql(
    `
    mutation($ownerId: ID!, $title: String!) {
      createProjectV2(input: { ownerId: $ownerId, title: $title }) {
        projectV2 {
          id
          title
          public
          url
          number
          closed
        }
      }
    }
    `,
    { ownerId, title },
  );
  const project = created.createProjectV2.projectV2;
  if (!project) {
    throw new Error("createProjectV2 returned no project.");
  }
  if (stayPrivate) {
    console.log(
      JSON.stringify({ project, visibility: "private" }, null, 2),
    );
    return;
  }
  const updated = await graphql(
    `
    mutation($projectId: ID!, $public: Boolean!) {
      updateProjectV2(input: { projectId: $projectId, public: $public }) {
        projectV2 {
          id
          title
          public
          url
          number
          closed
        }
      }
    }
    `,
    { projectId: project.id, public: true },
  );
  console.log(
    JSON.stringify(
      {
        project: updated.updateProjectV2.projectV2,
        visibility: "public",
      },
      null,
      2,
    ),
  );
}

async function projectData() {
  const { owner, repo } = repoParts();
  const number = projectNumber();
  return graphql(
    `
    query($owner:String!, $repo:String!, $number:Int!) {
      repository(owner:$owner, name:$repo) {
        issues(first:100, states:[OPEN, CLOSED]) {
          nodes {
            id
            number
            title
            state
          }
        }
      }
      user(login:$owner) {
        projectV2(number:$number) {
          id
          title
          closed
          fields(first:50) {
            nodes {
              ... on ProjectV2FieldCommon {
                id
                name
              }
              ... on ProjectV2SingleSelectField {
                id
                name
                options {
                  id
                  name
                  color
                  description
                }
              }
            }
          }
          items(first:100) {
            totalCount
            nodes {
              id
              fieldValues(first:20) {
                nodes {
                  ... on ProjectV2ItemFieldSingleSelectValue {
                    name
                    field {
                      ... on ProjectV2SingleSelectField {
                        name
                      }
                    }
                  }
                }
              }
              content {
                __typename
                ... on Issue {
                  id
                  number
                  title
                  state
                }
              }
            }
          }
        }
      }
    }
    `,
    { owner, repo, number },
  );
}


async function fetchAllNonPullRequestIssues() {
  const { owner, repo } = repoParts();
  const out = [];
  for (let page = 1; page <= 50; page += 1) {
    const batch = await rest(
      "GET",
      `/repos/${owner}/${repo}/issues?state=all&per_page=100&page=${page}`,
    );
    if (!batch.length) {
      break;
    }
    for (const issue of batch) {
      if (!issue.pull_request) {
        out.push(issue);
      }
    }
    if (batch.length < 100) {
      break;
    }
  }
  return out;
}

function normalizeTitleFilter(value) {
  return String(value).toLowerCase();
}

async function listIssues(options) {
  const raw = String(options.state || "all").toLowerCase();
  const titlePrefix = options["title-prefix"] || options.titlePrefix;
  const titleContains = options["title-contains"] || options.titleContains;
  const limit = Math.max(1, Math.min(2000, Number(options.limit || 500)));

  let issues = await fetchAllNonPullRequestIssues();
  if (raw === "open") {
    issues = issues.filter((issue) => issue.state === "open");
  } else if (raw === "closed") {
    issues = issues.filter((issue) => issue.state === "closed");
  } else if (raw !== "all") {
    throw new Error("list-issues --state must be open, closed, or all.");
  }
  if (titlePrefix) {
    const normalizedPrefix = normalizeTitleFilter(titlePrefix);
    issues = issues.filter((issue) => normalizeTitleFilter(issue.title).startsWith(normalizedPrefix));
  }
  if (titleContains) {
    const normalizedContains = normalizeTitleFilter(titleContains);
    issues = issues.filter((issue) => normalizeTitleFilter(issue.title).includes(normalizedContains));
  }

  issues = issues.slice(0, limit);
  console.log(
    JSON.stringify(
      issues.map((issue) => ({
        number: issue.number,
        state: issue.state,
        title: issue.title,
        url: issue.html_url,
        labels: (issue.labels || []).map((label) => label.name),
      })),
      null,
      2,
    ),
  );
}

async function getIssueCmd(options) {
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("get-issue requires --number <n>.");
  }
  const { owner, repo } = repoParts();
  const issue = await rest("GET", `/repos/${owner}/${repo}/issues/${number}`);
  console.log(
    JSON.stringify(
      {
        number: issue.number,
        title: issue.title,
        body: issue.body,
        state: issue.state,
        labels: (issue.labels || []).map((l) => l.name),
        milestone: issue.milestone ? { number: issue.milestone.number, title: issue.milestone.title } : null,
        assignees: (issue.assignees || []).map((a) => a.login),
        url: issue.html_url,
        isPullRequest: Boolean(issue.pull_request),
      },
      null,
      2,
    ),
  );
}

async function deleteIssueByNumber(options) {
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("delete-issue requires --number <n>.");
  }
  const { owner, repo } = repoParts();
  const issue = await rest("GET", `/repos/${owner}/${repo}/issues/${number}`);
  if (issue.pull_request) {
    throw new Error("delete-issue only deletes issues, not pull requests.");
  }
  await graphql(
    `
    mutation($issue: ID!) {
      deleteIssue(input: { issueId: $issue }) {
        clientMutationId
      }
    }
    `,
    { issue: issue.node_id },
  );
  console.log(JSON.stringify({ deleted: true, number }, null, 2));
}

async function labelIssue(options) {
  const { owner, repo } = repoParts();
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("label-issue requires --number <n>.");
  }
  const action = String(options.action || "").toLowerCase();

  if (action === "add") {
    const labels = parseCsvList(options.labels);
    if (labels.length === 0) {
      throw new Error("label-issue --action add requires --labels <a,b,...>.");
    }
    const issue = await rest("POST", `/repos/${owner}/${repo}/issues/${number}/labels`, { labels });
    console.log(JSON.stringify({ number, labels: issue.map((l) => l.name) }, null, 2));
    return;
  }

  if (action === "remove") {
    const label = String(options.label || "").trim();
    if (!label) {
      throw new Error("label-issue --action remove requires --label <name>.");
    }
    await rest("DELETE", `/repos/${owner}/${repo}/issues/${number}/labels/${encodeURIComponent(label)}`);
    console.log(JSON.stringify({ number, removed: label }, null, 2));
    return;
  }

  throw new Error("label-issue requires --action add (--labels <a,b>) or --action remove (--label <name>).");
}

function issueUrl(issueNumber) {
  const { owner, repo } = repoParts();
  return `https://github.com/${owner}/${repo}/issues/${issueNumber}`;
}

function pullUrl(pullNumber) {
  const { owner, repo } = repoParts();
  return `https://github.com/${owner}/${repo}/pull/${pullNumber}`;
}

function buildPullRequestBody(options, issue) {
  const sections = [];
  const taskTag = options.issue ? `Task: #${issue.number}` : null;
  if (taskTag) {
    sections.push(taskTag);
  }
  if (options.close && issue) {
    sections.push(`Closes #${issue.number}`);
  }
  if (options.body) {
    sections.push(options.body);
  } else {
    sections.push(
      [
        "## Summary",
        "",
        "- ",
        "",
        "## Verification",
        "",
        "- ",
      ].join("\n"),
    );
  }

  return sections.join("\n\n");
}

async function listPrs(options) {
  const state = options.state || "open";
  const pulls = await allPulls(state);
  console.log(
    JSON.stringify(
      pulls.map((pull) => ({
        number: pull.number,
        state: pull.state,
        draft: pull.draft,
        head: pull.head.ref,
        base: pull.base.ref,
        title: pull.title,
        url: pull.html_url,
      })),
      null,
      2,
    ),
  );
}

async function listComments(options) {
  const { owner, repo } = repoParts();
  const issueNumber = Number(options.issue);
  if (!Number.isInteger(issueNumber) || issueNumber <= 0) {
    throw new Error("issue-comment --action list requires --issue <n>.");
  }

  const comments = await rest(
    "GET",
    `/repos/${owner}/${repo}/issues/${issueNumber}/comments?per_page=100`,
  );

  console.log(
    JSON.stringify(
      comments.map((comment) => ({
        id: comment.id,
        user: comment.user?.login,
        created_at: comment.created_at,
        body: comment.body,
        url: comment.html_url,
      })),
      null,
      2,
    ),
  );
}

async function deleteComment(options) {
  const { owner, repo } = repoParts();
  const commentId = Number(options["comment-id"] || options.commentId);
  if (!Number.isInteger(commentId) || commentId <= 0) {
    throw new Error("issue-comment --action delete requires --comment-id <n>.");
  }

  await rest("DELETE", `/repos/${owner}/${repo}/issues/comments/${commentId}`);
  console.log(JSON.stringify({ deleted: true, commentId }, null, 2));
}

async function commentPr(options) {
  const { owner, repo } = repoParts();
  const number = Number(options.number);
  const body = options.body;

  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("comment-issue / comment-pr requires --number <n>.");
  }
  if (!body || typeof body !== "string") {
    throw new Error("comment-issue / comment-pr requires --body <text>.");
  }

  const comment = await rest(
    "POST",
    `/repos/${owner}/${repo}/issues/${number}/comments`,
    { body },
  );

  console.log(
    JSON.stringify(
      {
        id: comment.id,
        url: comment.html_url,
        created_at: comment.created_at,
      },
      null,
      2,
    ),
  );
}

async function ensureRelease(options) {
  const tag = options.tag;
  if (!tag || typeof tag !== "string") {
    throw new Error("ensure-release requires --tag <tag>.");
  }

  const existing = await getReleaseByTag(tag);
  if (existing) {
    console.log(
      JSON.stringify(
        {
          id: existing.id,
          tag: existing.tag_name,
          name: existing.name,
          draft: existing.draft,
          prerelease: existing.prerelease,
          url: existing.html_url,
          upload_url: existing.upload_url,
        },
        null,
        2,
      ),
    );
    return;
  }

  const created = await createRelease({
    tag,
    name: options.name,
    body: options.body,
    draft: Boolean(options.draft),
    prerelease: Boolean(options.prerelease),
  });

  console.log(
    JSON.stringify(
      {
        id: created.id,
        tag: created.tag_name,
        name: created.name,
        draft: created.draft,
        prerelease: created.prerelease,
        url: created.html_url,
        upload_url: created.upload_url,
      },
      null,
      2,
    ),
  );
}

async function uploadReleaseAsset(options) {
  const tag = options.tag;
  const filePath = options.file;
  if (!tag || typeof tag !== "string") {
    throw new Error("upload-release-asset requires --tag <tag>.");
  }
  if (!filePath || typeof filePath !== "string") {
    throw new Error("upload-release-asset requires --file <path>.");
  }

  const release = await getReleaseByTag(tag);
  if (!release) {
    throw new Error(`Release with tag '${tag}' does not exist. Run ensure-release first.`);
  }

  const existingAsset = (release.assets || []).find(
    (asset) => asset.name === path.basename(filePath),
  );
  if (existingAsset) {
    await deleteReleaseAsset(existingAsset.id);
  }

  const uploaded = await uploadReleaseAssetBinary(release.upload_url, filePath, options.label);
  console.log(
    JSON.stringify(
      {
        release: {
          id: release.id,
          tag: release.tag_name,
          url: release.html_url,
        },
        asset: {
          id: uploaded.id,
          name: uploaded.name,
          label: uploaded.label,
          size: uploaded.size,
          state: uploaded.state,
          download_url: uploaded.browser_download_url,
          url: uploaded.url,
        },
      },
      null,
      2,
    ),
  );
}

async function listAssets(options) {
  const tag = options.tag;
  if (!tag || typeof tag !== "string") {
    throw new Error("release-asset --action list requires --tag <tag>.");
  }

  const release = await getReleaseByTag(tag);
  if (!release) {
    throw new Error(`Release with tag '${tag}' does not exist.`);
  }

  console.log(
    JSON.stringify(
      {
        release: {
          id: release.id,
          tag: release.tag_name,
          url: release.html_url,
        },
        assets: (release.assets || []).map((asset) => ({
          id: asset.id,
          name: asset.name,
          label: asset.label,
          size: asset.size,
          state: asset.state,
          download_url: asset.browser_download_url,
          url: asset.url,
        })),
      },
      null,
      2,
    ),
  );
}

async function deleteAsset(options) {
  const assetId = Number(options["asset-id"] || options.assetId);
  if (!Number.isInteger(assetId) || assetId <= 0) {
    throw new Error("release-asset --action delete requires --asset-id <n>.");
  }

  await deleteReleaseAsset(assetId);
  console.log(JSON.stringify({ deleted: true, assetId }, null, 2));
}

async function commentPrVerification(options) {
  const number = Number(options.number);
  const artifactUrl = options["artifact-url"] || options.artifactUrl;
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("comment-pr-verification requires --number <n>.");
  }
  if (!artifactUrl || typeof artifactUrl !== "string") {
    throw new Error("comment-pr-verification requires --artifact-url <url>.");
  }

  const route = options.route || "/agents";
  const lines = [
    `Live ${route} verification passed.`,
    "",
    `- Artifact: ${artifactUrl}`,
  ];

  if (options["latest-session-count"] || options.latestSessionCount) {
    lines.push(
      `- Latest session cards detected: ${options["latest-session-count"] || options.latestSessionCount}`,
    );
  }
  if (options["connected-ribbon"] || options.connectedRibbon) {
    lines.push(
      `- Connected ribbon present: ${options["connected-ribbon"] || options.connectedRibbon}`,
    );
  }
  if (options.screenshot) {
    lines.push(`- Screenshot: ${options.screenshot}`);
  }
  if (options.dom) {
    lines.push(`- DOM snapshot: ${options.dom}`);
  }

  await commentPr({ number, body: lines.join("\n") });
}

async function mergePr(options) {
  const { owner, repo } = repoParts();
  const number = Number(options.number);
  const method = options.method || "merge";

  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("merge-pr requires --number <n>.");
  }
  if (!["merge", "squash", "rebase"].includes(method)) {
    throw new Error("merge-pr --method must be one of: merge, squash, rebase.");
  }

  const pull = await getPullRequest(number);
  const payload = {
    merge_method: method,
  };

  if (options.title) {
    payload.commit_title = options.title;
  }
  if (options.message) {
    payload.commit_message = options.message;
  }
  if (pull.head?.sha) {
    payload.sha = pull.head.sha;
  }

  const result = await rest("PUT", `/repos/${owner}/${repo}/pulls/${number}/merge`, payload);
  console.log(
    JSON.stringify(
      {
        merged: result.merged,
        message: result.message,
        sha: result.sha,
        pullRequest: {
          number,
          url: pullUrl(number),
        },
        method,
      },
      null,
      2,
    ),
  );
}

async function listReviewThreads(options) {
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("pr-review --action list requires --number <n>.");
  }

  const { owner, repo } = repoParts();
  const data = await graphql(
    `
    query($owner:String!, $repo:String!, $number:Int!) {
      repository(owner:$owner, name:$repo) {
        pullRequest(number:$number) {
          number
          title
          reviewThreads(first:100) {
            nodes {
              id
              isResolved
              isOutdated
              path
              line
              comments(first:20) {
                nodes {
                  id
                  author { login }
                  body
                  createdAt
                  url
                }
              }
            }
          }
        }
      }
    }
    `,
    { owner, repo, number },
  );

  const pr = data.repository.pullRequest;
  console.log(
    JSON.stringify(
      {
        number: pr.number,
        title: pr.title,
        threads: pr.reviewThreads.nodes.map((thread) => ({
          id: thread.id,
          isResolved: thread.isResolved,
          isOutdated: thread.isOutdated,
          path: thread.path,
          line: thread.line,
          comments: thread.comments.nodes.map((comment) => ({
            id: comment.id,
            author: comment.author?.login,
            body: comment.body,
            createdAt: comment.createdAt,
            url: comment.url,
          })),
        })),
      },
      null,
      2,
    ),
  );
}

async function resolveThread(options) {
  const threadId = options["thread-id"] || options.threadId;
  if (!threadId) {
    throw new Error("pr-review --action resolve requires --thread-id <id>.");
  }

  const data = await graphql(
    `
    mutation($threadId:ID!) {
      resolveReviewThread(input:{threadId:$threadId}) {
        thread {
          id
          isResolved
          isOutdated
          path
          line
        }
      }
    }
    `,
    { threadId },
  );

  console.log(JSON.stringify(data.resolveReviewThread.thread, null, 2));
}

async function createPr(options) {
  const { owner, repo } = repoParts();
  const head = options.head;
  const title = options.title;
  const base = options.base || "main";
  const issueNumber = options.issue ? Number(options.issue) : null;

  if (!head) {
    throw new Error("create-pr requires --head <branch>.");
  }
  if (!title) {
    throw new Error("create-pr requires --title <title>.");
  }
  if (options.issue && (!Number.isInteger(issueNumber) || issueNumber <= 0)) {
    throw new Error("create-pr requires a valid --issue <n> when provided.");
  }

  const issue = issueNumber ? await getIssue(issueNumber) : null;
  const body = buildPullRequestBody(
    {
      body: options.body,
      issue: issueNumber,
      close: Boolean(options.close || issueNumber),
    },
    issue,
  );

  const pull = await rest("POST", `/repos/${owner}/${repo}/pulls`, {
    head,
    base,
    title,
    body,
    draft: Boolean(options.draft),
  });

  if (issueNumber) {
    await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
      body: `Linked pull request: [#${pull.number}](${pull.html_url})`,
    });
  }

  console.log(
    JSON.stringify(
      {
        number: pull.number,
        draft: pull.draft,
        title: pull.title,
        head: pull.head.ref,
        base: pull.base.ref,
        url: pull.html_url,
      },
      null,
      2,
    ),
  );
}

async function updatePr(options) {
  const { owner, repo } = repoParts();
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("update-pr requires --number <n>.");
  }

  const patch = {};
  if (options.title) patch.title = options.title;
  if (options.body) patch.body = options.body;
  if (options.base) patch.base = options.base;
  if (options.state) patch.state = options.state;

  if (Object.keys(patch).length === 0) {
    throw new Error("update-pr requires at least one of --title, --body, --base, or --state.");
  }

  const pull = await rest("PATCH", `/repos/${owner}/${repo}/pulls/${number}`, patch);
  console.log(
    JSON.stringify(
      {
        number: pull.number,
        state: pull.state,
        draft: pull.draft,
        title: pull.title,
        head: pull.head.ref,
        base: pull.base.ref,
        url: pull.html_url,
      },
      null,
      2,
    ),
  );
}

async function createIssue(options) {
  const { owner, repo } = repoParts();
  const title = options.title != null ? String(options.title).trim() : "";
  if (!title) {
    throw new Error("create-issue requires --title <text>.");
  }

  const bodyFile = options["body-file"] || options.bodyFile;
  const hasInlineBody = options.body != null && String(options.body).length > 0;
  if (bodyFile && hasInlineBody) {
    throw new Error("create-issue: use either --body or --body-file, not both.");
  }
  let bodyText = "";
  if (bodyFile) {
    bodyText = await readFile(path.resolve(String(bodyFile)), "utf8");
  } else if (options.body != null) {
    bodyText = String(options.body);
  }

  const payload = {
    title,
    body: bodyText,
  };

  if (options.labels) {
    payload.labels = parseCsvList(options.labels);
  }

  if (options.milestone) {
    const milestone = Number(options.milestone);
    if (!Number.isInteger(milestone) || milestone <= 0) {
      throw new Error("create-issue --milestone must be a positive integer (repo milestone number).");
    }
    payload.milestone = milestone;
  }

  if (options.assignees) {
    payload.assignees = parseCsvList(options.assignees);
  }

  const issue = await rest("POST", `/repos/${owner}/${repo}/issues`, payload);
  console.log(
    JSON.stringify(
      {
        number: issue.number,
        state: issue.state,
        title: issue.title,
        url: issue.html_url,
        labels: issue.labels?.map((label) => label.name) || [],
      },
      null,
      2,
    ),
  );
}

async function updateIssue(options) {
  const { owner, repo } = repoParts();
  const number = Number(options.number);
  if (!Number.isInteger(number) || number <= 0) {
    throw new Error("update-issue requires --number <n>.");
  }

  const bodyFile = options["body-file"] || options.bodyFile;
  const hasInlineBody = options.body != null && String(options.body).length > 0;
  if (bodyFile && hasInlineBody) {
    throw new Error("update-issue: use either --body or --body-file, not both.");
  }

  const patch = {};
  if (options.title) patch.title = options.title;
  if (bodyFile) {
    patch.body = await readFile(path.resolve(String(bodyFile)), "utf8");
  } else if (options.body) {
    patch.body = options.body;
  }
  if (options.state) patch.state = options.state;
  if (options.labels) {
    patch.labels = parseCsvList(options.labels);
  }
  if (options.assignees) {
    patch.assignees = parseCsvList(options.assignees);
  }
  if (options.milestone !== undefined) {
    const milestone = Number(options.milestone);
    if (!Number.isInteger(milestone) || milestone <= 0) {
      throw new Error("update-issue --milestone must be a positive integer (repo milestone number).");
    }
    patch.milestone = milestone;
  }

  if (Object.keys(patch).length === 0) {
    throw new Error(
      "update-issue requires at least one of --title, --body, --body-file, --state, --labels, --assignees, or --milestone.",
    );
  }

  const issue = await rest("PATCH", `/repos/${owner}/${repo}/issues/${number}`, patch);
  console.log(
    JSON.stringify(
      {
        number: issue.number,
        state: issue.state,
        title: issue.title,
        url: issue.html_url,
      },
      null,
      2,
    ),
  );
}

async function linkPrTask(options) {
  const issueNumber = Number(options.issue);
  const pullNumber = Number(options.pr);
  if (!Number.isInteger(issueNumber) || issueNumber <= 0) {
    throw new Error("link-pr-task requires --issue <n>.");
  }
  if (!Number.isInteger(pullNumber) || pullNumber <= 0) {
    throw new Error("link-pr-task requires --pr <n>.");
  }

  const { patchedBody, postedComment } = await ensureIssueLinkedToPull(
    issueNumber,
    pullNumber,
    false,
  );

  console.log(
    JSON.stringify(
      {
        issue: {
          number: issueNumber,
          url: issueUrl(issueNumber),
        },
        pullRequest: {
          number: pullNumber,
          url: pullUrl(pullNumber),
        },
        closesIssue: Boolean(options.close),
        patchedBody,
        postedComment,
      },
      null,
      2,
    ),
  );
}

async function setIssueStatus(options) {
  const issuesArg = options.issues;
  const statusName = options.status;
  if (!issuesArg || !statusName) {
    throw new Error("set-issue-status requires --issues <n,n,...> and --status <status>.");
  }

  const requestedIssues = issuesArg
    .split(",")
    .map((value) => Number(value.trim()))
    .filter((value) => Number.isInteger(value) && value > 0);
  if (!requestedIssues.length) {
    throw new Error("No valid issue numbers were provided.");
  }

  const data = await projectData();
  const project = data.user.projectV2;
  const statusField = project.fields.nodes.find((field) => field?.name === "Status");
  if (!statusField?.options) {
    throw new Error("Project Status field was not found.");
  }

  const option = statusField.options.find((entry) => entry.name === statusName);
  if (!option) {
    throw new Error(`Unknown status option: ${statusName}`);
  }

  const projectItems = new Map(
    project.items.nodes
      .filter((item) => item.content?.__typename === "Issue")
      .map((item) => [item.content.number, item.id]),
  );

  const updated = [];
  for (const issueNumber of requestedIssues) {
    const itemId = projectItems.get(issueNumber);
    if (!itemId) {
      continue;
    }

    await graphql(
      `
      mutation($project:ID!, $item:ID!, $field:ID!, $option:String!) {
        updateProjectV2ItemFieldValue(input:{projectId:$project, itemId:$item, fieldId:$field, value:{singleSelectOptionId:$option}}) {
          projectV2Item { id }
        }
      }
      `,
      {
        project: project.id,
        item: itemId,
        field: statusField.id,
        option: option.id,
      },
    );
    updated.push(issueNumber);
  }

  console.log(
    JSON.stringify(
      {
        status: statusName,
        updatedIssues: updated,
      },
      null,
      2,
    ),
  );
}

async function report() {
  const data = await projectData();
  const project = data.user.projectV2;
  const milestones = await allMilestones();
  const issues = (await allIssues()).filter((issue) => !issue.pull_request);
  console.log(
    JSON.stringify(
      {
        repository: `${repoParts().owner}/${repoParts().repo}`,
        project: {
          title: project.title,
          number: projectNumber(),
          itemCount: project.items.totalCount,
          fields: project.fields.nodes
            .filter(Boolean)
            .map((field) => field.name),
        },
        issueCount: issues.length,
        milestones: milestones.map((milestone) => ({
          title: milestone.title,
          open: milestone.open_issues,
          closed: milestone.closed_issues,
        })),
      },
      null,
      2,
    ),
  );
}

function statusFromItem(item) {
  const values = item.fieldValues?.nodes || [];
  const statusValue = values.find(
    (value) => value?.field?.name === "Status" && value?.name,
  );
  return statusValue?.name || "Backlog";
}

function priorityFromItem(item) {
  const values = item.fieldValues?.nodes || [];
  const priorityValue = values.find(
    (value) => value?.field?.name === "Priority" && value?.name,
  );
  return priorityValue?.name || null;
}

function priorityRank(priority) {
  if (!priority) return 99;
  const normalized = priority.toLowerCase();
  if (normalized.includes("p0") || normalized === "high") return 0;
  if (normalized.includes("p1") || normalized === "medium") return 1;
  if (normalized.includes("p2") || normalized === "low") return 2;
  return 50;
}

async function listTasks(options) {
  const data = await projectData();
  const project = data.user.projectV2;
  const issues = await allIssues();
  const limit = Math.max(1, Number(options.limit || 10));
  const issueByNumber = new Map(
    issues
      .filter((issue) => !issue.pull_request)
      .map((issue) => [issue.number, issue]),
  );

  const candidates = project.items.nodes
    .map((item) => {
      const issue = item.content;
      if (!issue || issue.__typename !== "Issue") {
        return null;
      }
      const fullIssue = issueByNumber.get(issue.number);
      const status = statusFromItem(item);
      const priority = priorityFromItem(item);
      return {
        number: issue.number,
        title: issue.title,
        state: issue.state,
        status,
        priority,
        url: issueUrl(issue.number),
        labels: fullIssue?.labels?.map((label) => label.name) || [],
      };
    })
    .filter(Boolean)
    .filter((issue) => issue.state === "OPEN")
    .filter((issue) => issue.status !== "In Progress")
    .filter((issue) => issue.status !== "Done")
    .filter((issue) => issue.status !== "Ready for Merge");

  const readyCandidates = candidates.filter(
    (issue) => issue.status === "Ready" || issue.status === "Backlog",
  );

  const sorted = readyCandidates.sort((a, b) => {
    const priorityDiff = priorityRank(a.priority) - priorityRank(b.priority);
    if (priorityDiff !== 0) return priorityDiff;
    return a.number - b.number;
  });

  if (sorted.length === 0) {
    console.log(
      JSON.stringify(
        {
          message: "No Ready/Backlog tasks found that are not in progress.",
          candidateCount: candidates.length,
        },
        null,
        2,
      ),
    );
    return;
  }

  console.log(JSON.stringify(sorted.slice(0, limit), null, 2));
}

async function releaseAsset(options) {
  const action = String(options.action || "").toLowerCase();
  if (action === "list") {
    return listAssets(options);
  }
  if (action === "delete") {
    return deleteAsset(options);
  }
  throw new Error(
    "release-asset requires --action list (--tag <tag>) or --action delete (--asset-id <n>).",
  );
}

async function issueComment(options) {
  const action = String(options.action || "").toLowerCase();
  if (action === "list") {
    return listComments(options);
  }
  if (action === "delete") {
    return deleteComment(options);
  }
  throw new Error(
    "issue-comment requires --action list (--issue <n>) or --action delete (--comment-id <n>).",
  );
}

async function prReview(options) {
  const action = String(options.action || "").toLowerCase();
  if (action === "list") {
    return listReviewThreads(options);
  }
  if (action === "resolve") {
    return resolveThread(options);
  }
  throw new Error(
    "pr-review requires --action list (--number <n>) or --action resolve (--thread-id <id>).",
  );
}

async function project(options) {
  const action = String(options.action || "").toLowerCase();
  if (action === "link-prs") {
    return linkPrs(options);
  }
  if (action === "close-project") {
    return closeProject(options);
  }
  throw new Error(
    "project requires --action link-prs [--title-prefix <text>] [--dry-run] or --action close-project [--dry-run].",
  );
}

async function main(argv = process.argv.slice(2)) {
  const { command, options } = parseArgs(argv);

  if (!command || command === "help" || command === "--help") {
    usage();
    process.exit(command ? 0 : 1);
  }

  const commands = {
    "sync-labels": syncLabels,
    "list-issues": () => listIssues(options),
    "list-tasks": () => listTasks(options),
    "issue-comment": () => issueComment(options),
    "update-issue": () => updateIssue(options),
    "get-issue": () => getIssueCmd(options),
    "delete-issue": () => deleteIssueByNumber(options),
    "label-issue": () => labelIssue(options),
    "create-issue": () => createIssue(options),
    "create-project": () => createProjectCommand(options),
    "list-prs": () => listPrs(options),
    "comment-issue": () => commentPr(options),
    "comment-pr": () => commentPr(options),
    "ensure-release": () => ensureRelease(options),
    "upload-release-asset": () => uploadReleaseAsset(options),
    "release-asset": () => releaseAsset(options),
    "comment-pr-verification": () => commentPrVerification(options),
    "pr-review": () => prReview(options),
    "merge-pr": () => mergePr(options),
    "create-pr": () => createPr(options),
    "update-pr": () => updatePr(options),
    "link-pr-task": () => linkPrTask(options),
    "project": () => project(options),
    "set-project-visibility": () => setProjectVisibility(options),
    "set-issue-status": () => setIssueStatus(options),
    report,
  };

  if (!commands[command]) {
    usage();
    throw new Error(`Unknown command: ${command}`);
  }

  await commands[command]();
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}

export { linkPrTask, listIssues, main, parseArgs, updateIssue };
