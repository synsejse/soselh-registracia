export interface Session {
  id: number;
  field_code: string;
  field_name: string;
  session_date: string;
  start_time: string;
  end_time: string;
  max_capacity: number;
  turnus: number;
  available_spots: number;
}

export interface CreateRegistrationRequest {
  session_id: number;
  student_first_name: string;
  student_last_name: string;
  guardian_first_name: string;
  guardian_last_name: string;
  guardian_phone: string;
  guardian_email: string;
}

export interface RegistrationResponse {
  id: number;
  session: {
    id: number;
    field_code: string;
    field_name: string;
    session_date: string;
    start_time: string;
    end_time: string;
    max_capacity: number;
    turnus: number;
  };
  student_first_name: string;
  student_last_name: string;
  guardian_first_name: string;
  guardian_last_name: string;
  guardian_phone: string;
  guardian_email: string;
  confirmed: boolean;
  created_at: string;
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
    if (response.status === 412) {
      throw new ApiError(
        response.status,
        "Prihlasovanie nie je momentálne povolené",
      );
    }
    if (response.status === 409) {
      throw new ApiError(response.status, "Tento termín je už plný");
    }
    if (response.status === 404) {
      throw new ApiError(response.status, "Termín nebol nájdený");
    }
    throw new ApiError(response.status, response.statusText);
  }
  const text = await response.text();
  return text ? JSON.parse(text) : ({} as T);
}

export const api = {
  async getSessions(): Promise<Session[]> {
    const res = await fetch("/api/sessions");
    return handleResponse<Session[]>(res);
  },

  async createRegistration(data: CreateRegistrationRequest): Promise<number> {
    const res = await fetch("/api/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return handleResponse<number>(res);
  },

  async getRegistrationStatus(): Promise<boolean> {
    const res = await fetch("/api/status");
    return handleResponse<boolean>(res);
  },

  admin: {
    async login(password: string): Promise<void> {
      const res = await fetch("/api/admin/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ password }),
      });
      if (!res.ok) {
        throw new ApiError(res.status, "Nesprávne heslo");
      }
    },

    async logout(): Promise<void> {
      const res = await fetch("/api/admin/logout", {
        method: "POST",
      });
      return handleResponse<void>(res);
    },

    async checkAuth(): Promise<boolean> {
      const res = await fetch("/api/admin/check");
      return handleResponse<boolean>(res);
    },

    async getRegistrations(): Promise<RegistrationResponse[]> {
      const res = await fetch("/api/admin/registrations");
      return handleResponse<RegistrationResponse[]>(res);
    },

    async confirmRegistration(id: number): Promise<void> {
      const res = await fetch(`/api/admin/registrations/${id}/confirm`, {
        method: "POST",
      });
      if (!res.ok) throw new ApiError(res.status, "Nepodarilo sa potvrdiť registráciu");
    },

    async deleteRegistration(id: number): Promise<void> {
      const res = await fetch(`/api/admin/registrations/${id}`, {
        method: "DELETE",
      });
      if (!res.ok) throw new ApiError(res.status, "Nepodarilo sa zmazať registráciu");
    },

    async exportRegistrations(): Promise<void> {
      const res = await fetch("/api/admin/registrations/export");
      if (!res.ok) throw new ApiError(res.status, "Nepodarilo sa exportovať registrácie");

      // Handle file download
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `registrations-${new Date().toISOString().split('T')[0]}.xlsx`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    },

    async toggleRegistration(): Promise<boolean> {
      const res = await fetch("/api/admin/toggle", {
        method: "POST",
      });
      return handleResponse<boolean>(res);
    },
  },
};
