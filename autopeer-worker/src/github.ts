import type { OperationRecord } from "./types";
import { joinPath, readSecret, toBase64 } from "./utils";

interface GitHubRepoResponse {
  default_branch: string;
}

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

interface GitHubCheckRunsResponse {
  check_runs: Array<{
    name: string;
    status: string;
    conclusion: string | null;
    html_url?: string;
  }>;
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
      throw new Error(
        `GitHub API ${path} failed with HTTP ${response.status}: ${body}${this.errorHint(response.status)}`,
      );
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return (await response.json()) as T;
  }

  async getRepo(): Promise<GitHubRepoResponse> {
    return this.request<GitHubRepoResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}`,
    );
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
      throw new Error(
        `GitHub file read failed for ${path}: HTTP ${response.status}: ${body}${this.errorHint(response.status)}`,
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

  async listCheckRuns(ref: string): Promise<GitHubCheckRunsResponse> {
    return this.request<GitHubCheckRunsResponse>(
      `/repos/${joinPath(this.env.GITHUB_OWNER, this.env.GITHUB_REPO)}/commits/${encodeURIComponent(ref)}/check-runs`,
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
