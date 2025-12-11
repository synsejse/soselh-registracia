export interface Candidate {
  id: number;
  name: string;
}

export interface VotingStatus {
  ready: boolean;
  has_voted: boolean;
}

export interface SessionInfo {
  voter_id: string;
  name: string;
}

export interface CandidateResult {
  name: string;
  votes: number;
}

export interface LotteryWinner {
  name: string;
  voter_id: string;
}

export interface AdminStats {
  voted: number;
  unvoted: number;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new ApiError(response.status, response.statusText);
  }
  // Some endpoints might return empty body (e.g. 201 Created with no content, or 200 OK)
  // But our API mostly returns JSON.
  const text = await response.text();
  return text ? JSON.parse(text) : ({} as T);
}

export const api = {
  subscribeToStatus(callback: (status: VotingStatus) => void): WebSocket {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(
      `${protocol}//${window.location.host}/api/status/ws`,
    );

    ws.onmessage = (event) => {
      try {
        const status = JSON.parse(event.data);
        callback(status);
      } catch (e) {
        console.error("Failed to parse WS message", e);
      }
    };

    return ws;
  },

  async createSession(name: string): Promise<SessionInfo> {
    const res = await fetch("/api/session", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name }),
    });
    return handleResponse<SessionInfo>(res);
  },

  async getSession(): Promise<SessionInfo> {
    const res = await fetch("/api/session");
    return handleResponse<SessionInfo>(res);
  },

  async getCandidates(): Promise<Candidate[]> {
    const res = await fetch("/api/candidates");
    return handleResponse<Candidate[]>(res);
  },

  async castVote(candidateId: number): Promise<void> {
    const res = await fetch("/api/vote", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ candidate_id: candidateId }),
    });
    return handleResponse<void>(res);
  },

  admin: {
    async setStatus(action: "start" | "stop"): Promise<void> {
      const res = await fetch("/api/admin/status", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ action }),
      });
      return handleResponse<void>(res);
    },

    async getStats(): Promise<AdminStats> {
      const res = await fetch("/api/admin/stats");
      return handleResponse<AdminStats>(res);
    },

    async getResults(): Promise<CandidateResult[]> {
      const res = await fetch("/api/admin/results");
      return handleResponse<CandidateResult[]>(res);
    },

    async pickWinner(): Promise<LotteryWinner> {
      const res = await fetch("/api/admin/lottery");
      return handleResponse<LotteryWinner>(res);
    },

    async getStatus(): Promise<boolean> {
      const res = await fetch("/api/admin/status");
      return handleResponse<boolean>(res);
    },
  },
};
