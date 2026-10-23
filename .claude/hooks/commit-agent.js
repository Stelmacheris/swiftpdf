// SubagentStop hook: commit everything the subagent changed as "[agent-name] summary" and push to origin main.
const { execFileSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const MAX_SUBJECT_LENGTH = 72;
const TRAILER = "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>";

function readStdin() {
  try {
    return JSON.parse(fs.readFileSync(0, "utf8") || "{}");
  } catch {
    return {};
  }
}

function git(args, cwd) {
  return execFileSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
}

function report(message) {
  process.stdout.write(JSON.stringify({ systemMessage: message }));
}

function firstLineOf(text) {
  const line = (text || "").split("\n").map((l) => l.replace(/^[#*\-\s]+/, "").trim()).find(Boolean) || "";
  return line.length > MAX_SUBJECT_LENGTH ? line.slice(0, MAX_SUBJECT_LENGTH - 1) + "…" : line;
}

function buildSummary(input, changedFiles) {
  const fromMessage = firstLineOf(input.last_assistant_message);
  if (fromMessage) return fromMessage;
  const shown = changedFiles.slice(0, 3).join(", ");
  const more = changedFiles.length > 3 ? ` (+${changedFiles.length - 3} more)` : "";
  return `update ${shown}${more}`;
}

// The main session writes the role name here before launching an agent, because roles may run as general-purpose agents.
function readCurrentRole(cwd) {
  try {
    return fs.readFileSync(path.join(cwd, ".git", "claude-current-agent"), "utf8").trim();
  } catch {
    return "";
  }
}

function logInput(cwd, input) {
  try {
    const summary = { time: new Date().toISOString(), keys: Object.keys(input), agent_type: input.agent_type, hook_event_name: input.hook_event_name };
    fs.appendFileSync(path.join(cwd, ".git", "claude-hook-input.log"), JSON.stringify(summary) + "\n");
  } catch {}
}

function main() {
  const input = readStdin();
  const cwd = process.env.CLAUDE_PROJECT_DIR || input.cwd || process.cwd();
  logInput(cwd, input);
  // Internal helpers (e.g. progress summaries) also fire SubagentStop with an empty agent_type; only real agents commit.
  if (!input.agent_type) return;
  const agent = readCurrentRole(cwd) || input.agent_type || input.subagent_type || "agent";

  git(["add", "-A"], cwd);
  const changedFiles = git(["diff", "--cached", "--name-only"], cwd).split("\n").filter(Boolean);
  if (changedFiles.length === 0) return;

  const subject = `[${agent}] ${buildSummary(input, changedFiles)}`;
  const body = "Files:\n" + changedFiles.map((f) => `- ${f}`).join("\n");
  git(["commit", "-m", subject, "-m", body, "-m", TRAILER], cwd);
  const sha = git(["rev-parse", "--short", "HEAD"], cwd);

  const hasOrigin = git(["remote"], cwd).split("\n").includes("origin");
  if (!hasOrigin) {
    report(`Committed ${sha} ${subject} (no origin remote, not pushed)`);
    return;
  }
  try {
    git(["push", "origin", "HEAD:main"], cwd);
    report(`Committed and pushed ${sha} ${subject}`);
  } catch (error) {
    report(`Committed ${sha} ${subject} but push failed: ${String(error.stderr || error.message).trim()}`);
  }
}

try {
  main();
} catch (error) {
  report(`Auto-commit hook failed: ${String(error.stderr || error.message).trim()}`);
}
