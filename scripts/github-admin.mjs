// SPDX-License-Identifier: Apache-2.0

import { existsSync, readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

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
  task: { color: "ededed", description: "Tracked implementation task" },
  bug: { color: "d73a4a", description: "Something is not working" },
  enhancement: { color: "a2eeef", description: "New feature or improvement" },
};

const OLD_MILESTONE_TITLES = [
  "P0 Foundation Decisions",
  "P1 App Bootstrap",
  "P2 Server Function Backbone",
  "P3 OpenClaw Adapter Minimum Slice",
  "P4 Graph Assembly Service",
  "P5 Vertical UI Slice",
  "P6 Error Handling And Polish",
  "P7 End-To-End Proof",
];

const POC_MILESTONES = [
  {
    title: "POC V1 Foundation",
    description:
      "Core product framing, app bootstrap, and first end-to-end gateway/server-function connectivity. Covers the baseline shell, styling system, shared models, and the initial OpenClaw connection path.",
  },
  {
    title: "POC V1 Graph Backbone",
    description:
      "Data-layer work for turning OpenClaw state into graph-ready application data. Covers adapter slices, bindings, active sessions, relationship hints, graph semantics, and graph snapshot assembly.",
  },
  {
    title: "POC V1 Operator Experience",
    description:
      "UI completion, graph presentation, error handling, polish, and POC validation. Covers the dashboard and agents surface, visual graph work, refresh flows, smoke tests, and demo readiness.",
  },
];

const PHASE_PRIORITY = {
  "Foundation Decisions": "priority/p0",
  "App Bootstrap": "priority/p0",
  "Server Function Backbone": "priority/p0",
  "OpenClaw Adapter Minimum Slice": "priority/p0",
  "Graph Assembly Service": "priority/p1",
  "Vertical UI Slice": "priority/p1",
  "Error Handling And Polish": "priority/p2",
  "End-To-End Proof": "priority/p2",
};

const PHASE_MILESTONE = {
  "Foundation Decisions": "POC V1 Foundation",
  "App Bootstrap": "POC V1 Foundation",
  "Server Function Backbone": "POC V1 Foundation",
  "OpenClaw Adapter Minimum Slice": "POC V1 Graph Backbone",
  "Graph Assembly Service": "POC V1 Graph Backbone",
  "Vertical UI Slice": "POC V1 Operator Experience",
  "Error Handling And Polish": "POC V1 Operator Experience",
  "End-To-End Proof": "POC V1 Operator Experience",
};

const IMPLEMENTED_ISSUES = {
  5: {
    title: "T1.1 Create the Rust/Dioxus application skeleton",
    body:
      "the runnable Rust/Dioxus application skeleton\n- the crate manifest and Dioxus app metadata\n- `src/main.rs` app entrypoint\n- the initial router shell used by the app",
  },
  6: {
    title: "T1.2 Add the base route structure",
    body:
      "typed routes for `/`, `/agents`, and `/settings`\n- a shared application layout\n- persistent sidebar navigation and top bar\n- a safe not-found route",
  },
  7: {
    title: "T1.3 Add design tokens and global styling",
    body:
      "the Tailwind 4 styling pipeline\n- shared design tokens and custom mission-control styling\n- panel, typography, spacing, border, and surface treatments\n- the generated application stylesheet served through Dioxus",
  },
  9: {
    title: "T2.1 Add the first live gateway connectivity slice",
    body:
      "the first live Dioxus server-backed gateway slice\n- a server-side loopback WebSocket connection to the OpenClaw Gateway\n- a small typed gateway status payload rendered in the UI\n- degraded/error handling when config or connectivity fails",
  },
  12: {
    title: "T2.4 Add the first Dioxus server function",
    body:
      "the first Dioxus server function, `get_gateway_status()`\n- dashboard-side resource loading for that server function\n- user-visible loading, healthy, and degraded rendering states",
  },
  15: {
    title: "T3.2 Implement gateway configuration loading",
    body:
      "gateway config loading from `~/.openclaw/openclaw.json`\n- parsing of `gateway.port`\n- parsing of `gateway.auth.token`\n- actionable degraded states when config is missing or invalid",
  },
  16: {
    title: "T3.3 Implement `gateway_status()`",
    body:
      "a gateway health/status call over the documented WebSocket path\n- mapping into the shared `GatewayStatusSnapshot` model\n- healthy vs degraded UI presentation on the dashboard and status pill",
  },
  24: {
    title: "T5.1 Build the dashboard shell",
    body:
      "the dashboard route shell\n- the hero status area\n- the gateway status card\n- adjacent dashboard panels that reserve space for the graph-focused surface\n- loading/error/success states around the gateway-backed summary area",
  },
};

const T0_IMPLEMENTED_ISSUES = {
  1: {
    comment:
      "the Phase 0 foundation note in `docs/milestones/proof-of-concept-1/t0_foundation_decisions.md`\n- explicit node, edge, status, and provenance semantics for the POC graph\n- clear guidance on what the UI may and may not claim from the underlying data",
  },
  2: {
    comment:
      "the Phase 0 foundation note in `docs/milestones/proof-of-concept-1/t0_foundation_decisions.md`\n- the minimal `GatewayAdapter` capability contract for the first slice\n- the initial shared model surface the adapter should stabilize for later implementation work",
  },
  4: {
    comment:
      "the Phase 0 foundation note in `docs/milestones/proof-of-concept-1/t0_foundation_decisions.md`\n- the deterministic SVG rendering decision for POC V1\n- layout and verification constraints that keep the first graph slice stable and testable",
  },
};

function usage() {
  console.log(`Usage:
  node scripts/github-admin.mjs <command> [options]

Commands:
  sync-labels
  assign-poc [--assignee <login>]
  remap-poc-milestones
  reconcile-poc-doc
  find-task --task <Tn.n>
  list-prs [--state <open|closed|all>]
  create-pr --head <branch> --title <title> [--base <branch>] [--body <text>] [--issue <n>] [--draft]
  update-pr --number <n> [--title <title>] [--body <text>] [--base <branch>] [--state <open|closed>]
  link-pr-task --pr <n> --issue <n> [--close]
  set-project-status-workflow
  set-issue-status --issues <n,n,...> --status <status>
  repair-t0-numbering
  audit-poc-consistency
  complete-issue --issue <n> --commit <sha> --note <text>
  complete-t0 --commit <sha>
  close-implemented [--commit <sha>]
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

function parsePocTasks(markdown) {
  const lines = markdown.split("\n");
  let phase = null;
  const tasks = new Map();

  for (const line of lines) {
    if (line.startsWith("# Phase ")) {
      phase = line.split(":", 2)[1].trim();
      continue;
    }
    const match = line.match(/^## (T\d+\.\d+) (.+)$/);
    if (match && phase) {
      const title = `[POC V1] ${match[1]} ${match[2].trim()}`;
      tasks.set(title, { phase, taskId: match[1] });
    }
  }

  return tasks;
}

function taskIdFromTitle(title) {
  const match = title.match(/^\[POC V1\] (T\d+\.\d+) /);
  return match ? match[1] : null;
}

function parseNumberedSection(markdown, heading) {
  const escapedHeading = heading.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const match = markdown.match(
    new RegExp(`## ${escapedHeading}\\n\\n([\\s\\S]*?)(?:\\n## |\\n# |$)`),
  );
  if (!match) {
    return [];
  }

  return match[1]
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => /^\d+\.\s+T\d+\.\d+$/.test(line))
    .map((line) => {
      const [, number, taskId] = line.match(/^(\d+)\.\s+(T\d+\.\d+)$/);
      return { number: Number(number), taskId };
    });
}

function addUnique(list, item) {
  if (!list.includes(item)) {
    list.push(item);
  }
}

function labelsForTask(phase, title) {
  const labels = ["poc-v1", "task", PHASE_PRIORITY[phase]];
  const titleLower = title.toLowerCase();

  if (phase === "Foundation Decisions") {
    addUnique(labels, "design");
    if (titleLower.includes("graph")) addUnique(labels, "graph");
    if (titleLower.includes("adapter")) addUnique(labels, "adapter");
    if (titleLower.includes("fixture") || titleLower.includes("test")) addUnique(labels, "testing");
  } else if (phase === "App Bootstrap") {
    addUnique(labels, "frontend");
    if (titleLower.includes("test")) addUnique(labels, "testing");
  } else if (phase === "Server Function Backbone") {
    addUnique(labels, "backend");
    addUnique(labels, "gateway");
    if (titleLower.includes("event bridge")) addUnique(labels, "frontend");
  } else if (phase === "OpenClaw Adapter Minimum Slice") {
    addUnique(labels, "backend");
    addUnique(labels, "adapter");
    addUnique(labels, "gateway");
  } else if (phase === "Graph Assembly Service") {
    addUnique(labels, "backend");
    addUnique(labels, "graph");
  } else if (phase === "Vertical UI Slice") {
    addUnique(labels, "frontend");
    addUnique(labels, "graph");
  } else if (phase === "Error Handling And Polish") {
    addUnique(labels, "frontend");
    if (titleLower.includes("connection") || titleLower.includes("refresh")) addUnique(labels, "gateway");
  } else if (phase === "End-To-End Proof") {
    addUnique(labels, "testing");
    if (titleLower.includes("adapter")) addUnique(labels, "adapter");
  }

  return labels;
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

  const tasks = parsePocTasks(
    await readFile("docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md", "utf8"),
  );
  const issues = (await allIssues()).filter(
    (issue) => !issue.pull_request && tasks.has(issue.title),
  );

  for (const issue of issues) {
    const { phase } = tasks.get(issue.title);
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${issue.number}`, {
      labels: labelsForTask(phase, issue.title),
    });
  }

  console.log(`Synced ${Object.keys(LABEL_DEFS).length} labels and relabeled ${issues.length} POC issues.`);
}

async function ensureMilestones() {
  const { owner, repo } = repoParts();
  const existing = await allMilestones();
  const byTitle = new Map(existing.map((milestone) => [milestone.title, milestone]));
  const created = new Map();

  for (const spec of POC_MILESTONES) {
    if (byTitle.has(spec.title)) {
      created.set(
        spec.title,
        await rest("PATCH", `/repos/${owner}/${repo}/milestones/${byTitle.get(spec.title).number}`, {
          title: spec.title,
          description: spec.description,
          state: "open",
        }),
      );
    } else {
      created.set(spec.title, await rest("POST", `/repos/${owner}/${repo}/milestones`, spec));
    }
  }

  return created;
}

async function remapPocMilestones() {
  const { owner, repo } = repoParts();
  const tasks = parsePocTasks(
    await readFile("docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md", "utf8"),
  );
  const milestones = await ensureMilestones();
  const issues = (await allIssues()).filter(
    (issue) => !issue.pull_request && tasks.has(issue.title),
  );

  for (const issue of issues) {
    const { phase } = tasks.get(issue.title);
    const milestone = milestones.get(PHASE_MILESTONE[phase]);
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${issue.number}`, {
      milestone: milestone.number,
    });
  }

  const freshMilestones = await allMilestones();
  const stillUsed = new Set(
    (await allIssues()).filter((issue) => !issue.pull_request && issue.milestone).map((issue) => issue.milestone.title),
  );
  for (const milestone of freshMilestones) {
    if (OLD_MILESTONE_TITLES.includes(milestone.title) && !stillUsed.has(milestone.title)) {
      await rest("DELETE", `/repos/${owner}/${repo}/milestones/${milestone.number}`);
    }
  }

  console.log(`Reassigned ${issues.length} POC issues into ${POC_MILESTONES.length} milestones.`);
}

async function assignPoc(options) {
  const { owner, repo } = repoParts();
  const assignee = options.assignee || owner;
  const issues = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith("[POC V1] "),
  );

  for (const issue of issues) {
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${issue.number}`, {
      assignees: [assignee],
    });
  }

  console.log(`Assigned ${issues.length} POC issues to ${assignee}.`);
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

async function setProjectStatusWorkflow() {
  const data = await projectData();
  const project = data.user.projectV2;
  const statusField = project.fields.nodes.find((field) => field?.name === "Status");
  if (!statusField) {
    throw new Error("Project Status field was not found.");
  }

  const updated = await graphql(
    `
    mutation($field:ID!, $name:String!, $options:[ProjectV2SingleSelectFieldOptionInput!]) {
      updateProjectV2Field(input:{fieldId:$field, name:$name, singleSelectOptions:$options}) {
        projectV2Field {
          ... on ProjectV2SingleSelectField {
            id
            name
            options { id name }
          }
        }
      }
    }
    `,
    {
      field: statusField.id,
      name: "Status",
      options: [
        { name: "Backlog", color: "GRAY", description: "Known work that is not yet ready to start" },
        { name: "Ready", color: "BLUE", description: "Ready to pick up next" },
        { name: "In Progress", color: "YELLOW", description: "Actively being worked on" },
        { name: "Ready for Merge", color: "ORANGE", description: "Implementation is complete and awaiting merge" },
        { name: "Blocked", color: "RED", description: "Blocked by an external dependency or decision" },
        { name: "Done", color: "GREEN", description: "Completed work" },
      ],
    },
  );

  const options = Object.fromEntries(
    updated.updateProjectV2Field.projectV2Field.options.map((option) => [option.name, option.id]),
  );

  for (const item of project.items.nodes) {
    const issue = item.content;
    if (!issue || issue.__typename !== "Issue") {
      continue;
    }
    const target = issue.state === "CLOSED" ? "Done" : "Backlog";
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
        item: item.id,
        field: statusField.id,
        option: options[target],
      },
    );
  }

  console.log(
    "Updated Project Status workflow to Backlog / Ready / In Progress / Ready for Merge / Blocked / Done.",
  );
}

async function findTask(options) {
  const taskId = options.task;
  if (!taskId) {
    throw new Error("find-task requires --task <Tn.n>.");
  }

  const prefix = `[POC V1] ${taskId} `;
  const matches = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith(prefix),
  );

  console.log(
    JSON.stringify(
      matches.map((issue) => ({
        number: issue.number,
        state: issue.state,
        title: issue.title,
      })),
      null,
      2,
    ),
  );
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
  const taskTag = options.issue
    ? `Task: #${issue.number} ${issue.title}`
    : null;
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

async function linkPrTask(options) {
  const { owner, repo } = repoParts();
  const issueNumber = Number(options.issue);
  const pullNumber = Number(options.pr);
  if (!Number.isInteger(issueNumber) || issueNumber <= 0) {
    throw new Error("link-pr-task requires --issue <n>.");
  }
  if (!Number.isInteger(pullNumber) || pullNumber <= 0) {
    throw new Error("link-pr-task requires --pr <n>.");
  }

  const pull = await getPullRequest(pullNumber);
  const closingLine = options.close ? `\n\nCloses #${issueNumber}` : "";
  const nextBody = `${pull.body || ""}${closingLine}`.trim();

  await rest("PATCH", `/repos/${owner}/${repo}/pulls/${pullNumber}`, {
    body: nextBody,
  });
  await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
    body: `Linked pull request: [#${pullNumber}](${pullUrl(pullNumber)})`,
  });

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
      },
      null,
      2,
    ),
  );
}

async function reconcilePocDoc() {
  const { owner, repo } = repoParts();
  const markdown = await readFile(
    "docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md",
    "utf8",
  );
  const tasks = parsePocTasks(markdown);
  const milestones = await ensureMilestones();
  const issues = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith("[POC V1] "),
  );
  const byTitle = new Map(issues.map((issue) => [issue.title, issue]));
  const byTaskId = new Map(
    issues
      .map((issue) => [taskIdFromTitle(issue.title), issue])
      .filter(([taskId]) => taskId),
  );

  const project = (await projectData()).user.projectV2;
  const statusField = project.fields.nodes.find((field) => field?.name === "Status");
  const backlogOption = statusField?.options?.find((option) => option.name === "Backlog");
  const projectItems = new Map(
    project.items.nodes
      .filter((item) => item.content?.__typename === "Issue")
      .map((item) => [item.content.number, item.id]),
  );

  const created = [];
  const renamed = [];

  for (const [title, meta] of tasks.entries()) {
    let issue = byTitle.get(title);
    if (!issue) {
      const taskId = meta.taskId;
      const existingByTaskId = byTaskId.get(taskId);
      if (existingByTaskId && existingByTaskId.title !== title) {
        issue = await rest("PATCH", `/repos/${owner}/${repo}/issues/${existingByTaskId.number}`, {
          title,
        });
        renamed.push({
          number: issue.number,
          from: existingByTaskId.title,
          to: title,
        });
        byTitle.delete(existingByTaskId.title);
        byTitle.set(title, issue);
        byTaskId.set(taskId, issue);
      }
    }

    if (!issue) {
      issue = await rest("POST", `/repos/${owner}/${repo}/issues`, {
        title,
        labels: labelsForTask(meta.phase, title),
        milestone: milestones.get(PHASE_MILESTONE[meta.phase]).number,
        assignees: [owner],
      });
      created.push({ number: issue.number, title: issue.title });
      byTitle.set(title, issue);
      byTaskId.set(meta.taskId, issue);
    } else {
      await rest("PATCH", `/repos/${owner}/${repo}/issues/${issue.number}`, {
        labels: labelsForTask(meta.phase, title),
        milestone: milestones.get(PHASE_MILESTONE[meta.phase]).number,
      });
    }

    let itemId = projectItems.get(issue.number);
    if (!itemId) {
      itemId = await addIssueToProject(project.id, issue.node_id);
      projectItems.set(issue.number, itemId);
    }

    if (statusField && backlogOption && issue.state === "open") {
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
          option: backlogOption.id,
        },
      );
    }
  }

  const stale = issues
    .filter((issue) => !tasks.has(issue.title) && !tasks.has(`[POC V1] ${taskIdFromTitle(issue.title) || ""}`))
    .map((issue) => ({
      number: issue.number,
      title: issue.title,
    }));

  console.log(
    JSON.stringify(
      {
        created,
        renamed,
        stale,
      },
      null,
      2,
    ),
  );
}

async function closeImplemented(options) {
  const { owner, repo } = repoParts();
  const commit = options.commit || "062d615";
  const commitUrl = `https://github.com/${owner}/${repo}/commit/${commit}`;

  for (const [issueNumber, issue] of Object.entries(IMPLEMENTED_ISSUES)) {
    await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
      body: `Implemented in [${commit}](${commitUrl}).\n\nWhat landed:\n- ${issue.body}\n\nThis is now present in the current repository state and was shipped as part of the initial published baseline.`,
    });
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${issueNumber}`, { state: "closed" });
  }

  console.log(`Commented on and closed ${Object.keys(IMPLEMENTED_ISSUES).length} implemented issues.`);
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

async function completeT0(options) {
  const { owner, repo } = repoParts();
  const commit = options.commit;
  if (!commit) {
    throw new Error("complete-t0 requires --commit <sha>.");
  }
  const commitUrl = `https://github.com/${owner}/${repo}/commit/${commit}`;

  for (const [issueNumber, issue] of Object.entries(T0_IMPLEMENTED_ISSUES)) {
    await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
      body:
        `Implemented in [${commit}](${commitUrl}).\n\nWhat landed:\n- ${issue.comment}\n\nThis completes the corresponding T0 foundation decision work in the repository docs.`,
    });
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${issueNumber}`, {
      state: "closed",
    });
  }

  await setIssueStatus({
    issues: Object.keys(T0_IMPLEMENTED_ISSUES).join(","),
    status: "Done",
  });

  console.log(
    `Commented on, closed, and marked done for ${Object.keys(T0_IMPLEMENTED_ISSUES).length} T0 issues.`,
  );
}

async function completeIssue(options) {
  const { owner, repo } = repoParts();
  const issueNumber = Number(options.issue);
  const commit = options.commit;
  const note = options.note;

  if (!Number.isInteger(issueNumber) || issueNumber <= 0) {
    throw new Error("complete-issue requires --issue <n>.");
  }
  if (!commit) {
    throw new Error("complete-issue requires --commit <sha>.");
  }
  if (!note) {
    throw new Error("complete-issue requires --note <text>.");
  }

  const commitUrl = `https://github.com/${owner}/${repo}/commit/${commit}`;
  await rest("POST", `/repos/${owner}/${repo}/issues/${issueNumber}/comments`, {
    body:
      `Implemented in [${commit}](${commitUrl}).\n\nWhat landed:\n- ${note}\n\nThis issue is now complete in the repository state referenced above.`,
  });
  await rest("PATCH", `/repos/${owner}/${repo}/issues/${issueNumber}`, {
    state: "closed",
  });

  await setIssueStatus({
    issues: String(issueNumber),
    status: "Done",
  });

  console.log(`Commented on, closed, and marked done for issue #${issueNumber}.`);
}

async function repairT0Numbering() {
  const { owner, repo } = repoParts();
  const issues = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith("[POC V1] T0."),
  );

  const removed = issues.find(
    (issue) => issue.title === "[POC V1] T0.3 Create stable fixtures before adapter implementation",
  );
  const renamed = issues.find(
    (issue) => issue.title === "[POC V1] T0.4 Choose the graph rendering strategy",
  );

  if (renamed) {
    await rest("PATCH", `/repos/${owner}/${repo}/issues/${renamed.number}`, {
      title: "[POC V1] T0.3 Choose the graph rendering strategy",
    });
  }

  if (removed) {
    await graphql(
      `
      mutation($issue:ID!) {
        deleteIssue(input:{issueId:$issue}) {
          clientMutationId
        }
      }
      `,
      { issue: removed.node_id },
    );
  }

  const refreshed = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith("[POC V1] T0."),
  );

  console.log(
    JSON.stringify(
      {
        deletedIssue: removed ? { number: removed.number, title: removed.title } : null,
        renamedIssue: renamed
          ? {
              number: renamed.number,
              title: "[POC V1] T0.3 Choose the graph rendering strategy",
            }
          : null,
        currentT0Issues: refreshed.map((issue) => ({
          number: issue.number,
          state: issue.state,
          title: issue.title,
        })),
      },
      null,
      2,
    ),
  );
}

async function auditPocConsistency() {
  const markdown = await readFile(
    "docs/milestones/proof-of-concept-1/poc_v1_task_breakdown.md",
    "utf8",
  );
  const tasks = parsePocTasks(markdown);
  const docTitles = [...tasks.keys()].sort();

  const suggestedExecutionOrder = parseNumberedSection(markdown, "Suggested Execution Order");
  const smallestUsefulVerticalSlice = parseNumberedSection(markdown, "Smallest Useful Vertical Slice");

  function numberingIssues(entries, sectionName) {
    const problems = [];
    for (let index = 0; index < entries.length; index += 1) {
      const expected = index + 1;
      if (entries[index].number !== expected) {
        problems.push({
          section: sectionName,
          expected,
          found: entries[index].number,
          taskId: entries[index].taskId,
        });
      }
    }
    return problems;
  }

  const githubIssues = (await allIssues()).filter(
    (issue) => !issue.pull_request && issue.title.startsWith("[POC V1] "),
  );
  const githubTitles = githubIssues.map((issue) => issue.title).sort();

  const docMissingOnGithub = docTitles.filter((title) => !githubTitles.includes(title));
  const githubMissingInDoc = githubTitles.filter((title) => !docTitles.includes(title));

  const duplicateGithubTitles = [...new Set(
    githubTitles.filter((title, index) => githubTitles.indexOf(title) !== index),
  )];

  const t0Issues = githubIssues
    .filter((issue) => issue.title.startsWith("[POC V1] T0."))
    .map((issue) => ({ number: issue.number, title: issue.title }))
    .sort((a, b) => a.number - b.number);

  console.log(
    JSON.stringify(
      {
        local: {
          docTaskCount: docTitles.length,
          numberingProblems: [
            ...numberingIssues(suggestedExecutionOrder, "Suggested Execution Order"),
            ...numberingIssues(smallestUsefulVerticalSlice, "Smallest Useful Vertical Slice"),
          ],
        },
        github: {
          pocIssueCount: githubIssues.length,
          t0Issues,
          docMissingOnGithub,
          githubMissingInDoc,
          duplicateGithubTitles,
        },
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
  const issues = (await allIssues()).filter((issue) => !issue.pull_request && issue.title.startsWith("[POC V1] "));
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
        pocIssueCount: issues.length,
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

const { command, options } = parseArgs(process.argv.slice(2));

if (!command || command === "help" || command === "--help") {
  usage();
  process.exit(command ? 0 : 1);
}

const commands = {
  "sync-labels": syncLabels,
  "assign-poc": () => assignPoc(options),
  "remap-poc-milestones": remapPocMilestones,
  "reconcile-poc-doc": reconcilePocDoc,
  "find-task": () => findTask(options),
  "list-prs": () => listPrs(options),
  "create-pr": () => createPr(options),
  "update-pr": () => updatePr(options),
  "link-pr-task": () => linkPrTask(options),
  "set-project-status-workflow": setProjectStatusWorkflow,
  "set-issue-status": () => setIssueStatus(options),
  "repair-t0-numbering": repairT0Numbering,
  "audit-poc-consistency": auditPocConsistency,
  "complete-issue": () => completeIssue(options),
  "complete-t0": () => completeT0(options),
  "close-implemented": () => closeImplemented(options),
  report,
};

if (!commands[command]) {
  usage();
  throw new Error(`Unknown command: ${command}`);
}

await commands[command]();
