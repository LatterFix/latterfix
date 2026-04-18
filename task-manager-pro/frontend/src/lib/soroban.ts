/**
 * Soroban RPC Client for TaskManagerPro Smart Contract
 * Connects to a Soroban RPC endpoint to query task/bounty data from the contract.
 */

export interface Bounty {
  id: number;
  title: string;
  description: string;
  reward: number;
  token: string;
  difficulty: "Easy" | "Medium" | "Hard";
  tags: string[];
  postedAt: string;
  applicantCount: number;
  location: string;
  status: "Open" | "InProgress" | "Completed" | "Disputed";
  createdBy: string;
}

// Map contract TaskStatus to UI difficulty tiers based on reward ranges
function mapRewardToDifficulty(reward: number): Bounty["difficulty"] {
  if (reward < 400) return "Easy";
  if (reward < 800) return "Medium";
  return "Hard";
}

interface RawTask {
  id: { value: number };
  title: { to_string(): string };
  description: { to_string(): string };
  reward: { value: bigint };
  assignee: { high: number; low: number; signed: boolean } | null;
  status: { value: number };
  created_by: { high: number; low: number; signed: boolean };
  tags: { get(i: number): { to_string(): string }; len(): number };
}

interface SorobanRpcConfig {
  rpcUrl: string;
  contractId: string;
}

function getConfig(): SorobanRpcConfig {
  return {
    rpcUrl: process.env.SOROBAN_RPC_URL || "http://localhost:8000",
    contractId: process.env.SOROBAN_CONTRACT_ID || "",
  };
}

function parseAddress(
  addr: { high: number; low: number; signed: boolean } | null
): string {
  if (!addr) return "";
  const highHex = (addr.high >>> 0).toString(16).padStart(8, "0");
  const lowHex = (addr.low >>> 0).toString(16).padStart(8, "0");
  return `0x${highHex}${lowHex}`.toLowerCase();
}

function parseVecStrings(
  vec: { get(i: number): { to_string(): string }; len(): number }
): string[] {
  const len = vec.len();
  const result: string[] = [];
  for (let i = 0; i < len; i++) {
    result.push(vec.get(i).to_string());
  }
  return result;
}

/**
 * Fetch total bounty count from the contract.
 */
export async function fetchTaskCount(): Promise<number> {
  const { rpcUrl, contractId } = getConfig();
  if (!contractId) return 0;

  try {
    const response = await fetch(`${rpcUrl}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 3,
        method: "contract_call",
        params: {
          contract_id: contractId,
          method: "get_task_count",
          args: [],
        },
      }),
    });

    const data = await response.json();
    if (!data.result?.value) return 0;

    const values = data.result.value as Array<{ value: number }>;
    const [val] = values;
    return val?.value ?? 0;
  } catch {
    return 0;
  }
}

/**
 * Fetch paginated tasks from the Soroban contract.
 */
export async function fetchTasks(
  startId = 0,
  limit = 20
): Promise<Bounty[]> {
  const { rpcUrl, contractId } = getConfig();
  if (!contractId) return [];

  try {
    const response = await fetch(`${rpcUrl}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 2,
        method: "contract_call",
        params: {
          contract_id: contractId,
          method: "get_tasks",
          args: [startId, limit],
        },
      }),
    });

    const data = await response.json();
    if (!data.result?.value) return [];

    const values = Array.isArray(data.result.value)
      ? data.result.value
      : [data.result.value];

    return values
      .map((v: unknown) => {
        try {
          return normalizeTask(v as RawTask);
        } catch {
          return null;
        }
      })
      .filter(Boolean) as Bounty[];
  } catch {
    return [];
  }
}

function normalizeTask(task: RawTask): Bounty {
  const STATUS_NAMES = ["Open", "InProgress", "Completed", "Disputed"] as const;
  const rewardNum =
    typeof task.reward?.value === "bigint"
      ? Number(task.reward.value)
      : Number(task.reward?.value ?? 0);

  return {
    id: task.id?.value ?? 0,
    title: task.title?.to_string?.() ?? "",
    description: task.description?.to_string?.() ?? "",
    reward: rewardNum,
    token: "USDC",
    difficulty: mapRewardToDifficulty(rewardNum),
    tags: parseVecStrings(task.tags as { get(i: number): { to_string(): string }; len(): number }),
    postedAt: "0d ago",
    applicantCount: 0,
    location: "Remote",
    status: STATUS_NAMES[task.status?.value ?? 0] as Bounty["status"],
    createdBy: parseAddress(task.created_by),
  };
}
