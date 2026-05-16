import type { OperationRecord } from "./types";
import { HttpError, joinPath, readSecret, toBase64, uiMessage } from "./utils";

interface GitHubRefResponse {
  object: { sha: string };
}

interface GitHubFileResponse {
  sha: string;
  content?: string;
  encoding?: string;
}

interface GitHubPrResponse {
  number: number;
  html_url: string;
  node_id: string;
  state: string;
  merged: boolean;
  merged_at: string | null;
  merge_commit_sha: string | null;
  head: { sha: string };
}

export interface GitHubWorkflowRun {
  id: number;
  html_url: string;
  status: string;
  conclusion: string | null;
  head_sha: string;
  event: string;
  created_at: string;
  updated_at: string;
}

interface GitHubWorkflowRunsResponse {
  workflow_runs: GitHubWorkflowRun[];
}

export interface GitHubWorkflowJobStep {
  name: string;
  status: string;
  conclusion: string | null;
  number: number;
}

export interface GitHubWorkflowJob {
  id: number;
  name: string;
  status: string;
  conclusion: string | null;
  html_url: string;
  steps?: GitHubWorkflowJobStep[];
}

interface GitHubWorkflowJobsResponse {
  jobs: GitHubWorkflowJob[];
}

interface GitHubCheckRunAnnotation {
  path: string;
  start_line: number;
  end_line: number;
  annotation_level: string;
  title: string | null;
  message: string;
  raw_details: string | null;
}

interface GitHubMergeResponse {
  sha: string;
  merged: boolean;
  message: string;
}

export interface GitHubFileContent {
  exists: boolean;
  sha?: string;
  text?: string;
}

const GITHUB_USER_AGENT = "bird-lg-rs-autopeer-worker";

export class GitHubClient {
  constructor(private readonly env: Env) {}

  private errorHint(status: number): string {
    if (status !== 403) {
      return "";
    }
    return " Hint: GitHub requires a valid User-Agent header on all API requests. GITHUB_TOKEN must also have the repository permissions required by this API call. For this workflow, use a fine-grained token scoped to the target repo with at least Metadata: read, Contents: read/write, Pull requests: read/write, and Actions: read. For fine-grained PATs, the target repository must be owned by the token owner or by an organization that the token owner is a member of.";
  }

  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const response = await fetch(`https://api.github.com${path}`, {
      ...init,
      headers: {
        accept: "application/vnd.github+json",
        authorization: `Bearer ${readSecret(this.env, "GITHUB_TOKEN")}`,
        "content-type": "application/json",
        "user-agent": GITHUB_USER_AGENT,
        "x-github-api-version": "2022-11-28",
        ...(init?.headers ?? {}),
      },
    });

    if (!response.ok) {
      const body = await response.text();
      console.error(
        `GitHub API ${path} failed with HTTP ${response.status}: ${body}${this.errorHint(response.status)}`,
      );
      throw new HttpError(
        uiMessage("error.github.api_failed", {
          path,
          status: String(response.status),
        }),
        502,
      );
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return (await response.json()) as T;
  }

  async getBranchHead(branch: string): Promise<string> {
    const ref = await this.request<GitHubRefResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/git/ref/heads/${encodeURIComponent(branch)}`,
    );
    return ref.object.sha;
  }

  async getFile(path: string, ref: string): Promise<GitHubFileContent> {
    const response = await fetch(
      `https://api.github.com/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/contents/${joinPath(path)}?ref=${encodeURIComponent(ref)}`,
      {
        headers: {
          accept: "application/vnd.github+json",
          authorization: `Bearer ${readSecret(this.env, "GITHUB_TOKEN")}`,
          "user-agent": GITHUB_USER_AGENT,
          "x-github-api-version": "2022-11-28",
        },
      },
    );

    if (response.status === 404) {
      return { exists: false };
    }
    if (!response.ok) {
      const body = await response.text();
      console.error(
        `GitHub file read failed for ${path}: HTTP ${response.status}: ${body}${this.errorHint(response.status)}`,
      );
      throw new HttpError(
        uiMessage("error.github.file_read_failed", {
          path,
          status: String(response.status),
        }),
        502,
      );
    }

    const body = (await response.json()) as GitHubFileResponse;
    return {
      exists: true,
      sha: body.sha,
      text:
        body.encoding === "base64" && typeof body.content === "string"
          ? atob(body.content.replace(/\s+/g, ""))
          : "",
    };
  }

  async createBranch(branch: string, sha: string): Promise<void> {
    await this.request(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/git/refs`,
      {
        method: "POST",
        body: JSON.stringify({
          ref: `refs/heads/${branch}`,
          sha,
        }),
      },
    );
  }

  async forcePushSingleFile(input: {
    branch: string;
    baseSha: string;
    path: string;
    content: string;
    message: string;
  }): Promise<string> {
    const repoPath = joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO);

    const baseCommit = await this.request<{ tree: { sha: string } }>(
      `/repos/${repoPath}/git/commits/${encodeURIComponent(input.baseSha)}`,
    );

    const blob = await this.request<{ sha: string }>(
      `/repos/${repoPath}/git/blobs`,
      {
        method: "POST",
        body: JSON.stringify({ content: toBase64(input.content), encoding: "base64" }),
      },
    );

    const tree = await this.request<{ sha: string }>(
      `/repos/${repoPath}/git/trees`,
      {
        method: "POST",
        body: JSON.stringify({
          base_tree: baseCommit.tree.sha,
          tree: [
            { path: input.path, mode: "100644", type: "blob", sha: blob.sha },
          ],
        }),
      },
    );

    const commit = await this.request<{ sha: string }>(
      `/repos/${repoPath}/git/commits`,
      {
        method: "POST",
        body: JSON.stringify({
          message: input.message,
          tree: tree.sha,
          parents: [input.baseSha],
        }),
      },
    );

    await this.request(
      `/repos/${repoPath}/git/refs/heads/${input.branch}`,
      {
        method: "PATCH",
        body: JSON.stringify({ sha: commit.sha, force: true }),
      },
    );

    return commit.sha;
  }

  async upsertFile(input: {
    path: string;
    branch: string;
    sha?: string;
    content: string;
    message: string;
  }): Promise<void> {
    await this.request(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/contents/${joinPath(input.path)}`,
      {
        method: "PUT",
        body: JSON.stringify({
          message: input.message,
          branch: input.branch,
          sha: input.sha,
          content: toBase64(input.content),
        }),
      },
    );
  }

  async createPullRequest(input: {
    title: string;
    body: string;
    head: string;
    base: string;
  }): Promise<GitHubPrResponse> {
    return this.request<GitHubPrResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/pulls`,
      {
        method: "POST",
        body: JSON.stringify({
          title: input.title,
          body: input.body,
          head: input.head,
          base: input.base,
        }),
      },
    );
  }

  async mergePullRequest(number: number, sha: string): Promise<GitHubMergeResponse> {
    return this.request<GitHubMergeResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/pulls/${number}/merge`,
      {
        method: "PUT",
        body: JSON.stringify({
          sha,
          merge_method: "squash",
        }),
      },
    );
  }

  async getPullRequest(number: number): Promise<GitHubPrResponse> {
    return this.request<GitHubPrResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/pulls/${number}`,
    );
  }

  async createIssueComment(number: number, body: string): Promise<void> {
    await this.request<unknown>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/issues/${number}/comments`,
      {
        method: "POST",
        body: JSON.stringify({ body }),
      },
    );
  }

  async closePullRequest(number: number): Promise<GitHubPrResponse> {
    return this.request<GitHubPrResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/pulls/${number}`,
      {
        method: "PATCH",
        body: JSON.stringify({ state: "closed" }),
      },
    );
  }

  async deleteBranch(branch: string): Promise<void> {
    await this.request<void>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/git/refs/heads/${encodeURIComponent(branch)}`,
      { method: "DELETE" },
    );
  }

  async listWorkflowRuns(
    workflowId: string,
    options: {
      branch?: string;
      event?: string;
      perPage?: number;
    } = {},
  ): Promise<GitHubWorkflowRunsResponse> {
    const params = new URLSearchParams();
    if (options.branch) {
      params.set("branch", options.branch);
    }
    if (options.event) {
      params.set("event", options.event);
    }
    params.set("per_page", String(options.perPage ?? 20));

    return this.request<GitHubWorkflowRunsResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/actions/workflows/${encodeURIComponent(workflowId)}/runs?${params.toString()}`,
    );
  }

  async listWorkflowRunJobs(runId: number): Promise<GitHubWorkflowJobsResponse> {
    return this.request<GitHubWorkflowJobsResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/actions/runs/${runId}/jobs?per_page=100`,
    );
  }

  async listCheckRunAnnotations(checkRunId: number): Promise<GitHubCheckRunAnnotation[]> {
    return this.request<GitHubCheckRunAnnotation[]>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/check-runs/${checkRunId}/annotations?per_page=20`,
    );
  }

  async downloadJobLog(jobId: number): Promise<string | null> {
    try {
      const response = await fetch(
        `https://api.github.com/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/actions/jobs/${jobId}/logs`,
        {
          headers: {
            accept: "application/vnd.github+json",
            authorization: `Bearer ${readSecret(this.env, "GITHUB_TOKEN")}`,
            "user-agent": GITHUB_USER_AGENT,
            "x-github-api-version": "2022-11-28",
          },
          redirect: "follow",
        },
      );
      if (!response.ok) return null;
      return await response.text();
    } catch {
      return null;
    }
  }

  async dispatchWorkflow(
    workflowId: string,
    input: {
      ref: string;
      inputs?: Record<string, string>;
    },
  ): Promise<void> {
    await this.request<void>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/actions/workflows/${encodeURIComponent(workflowId)}/dispatches`,
      {
        method: "POST",
        body: JSON.stringify({
          ref: input.ref,
          inputs: input.inputs ?? {},
        }),
      },
    );
  }
}

export function branchName(operation: Pick<OperationRecord, "asn" | "node" | "kind" | "id">): string {
  return `autopeer/${operation.asn}/${operation.node}/${operation.kind}/${operation.id.slice(0, 8)}`;
}
