import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ServiceStatus {
  name: string;
  running: boolean;
  port: number | null;
  pid: number | null;
  uptime_secs: number | null;
  php_version: string | null;
}

interface ServiceState {
  services: ServiceStatus[];
  logs: string[];
  loading: boolean;
  error: string | null;

  fetchStatus: () => Promise<void>;
  startServices: () => Promise<void>;
  stopServices: () => Promise<void>;
  addLog: (line: string) => void;
  clearLogs: () => void;
  clearError: () => void;
  initListeners: () => Promise<() => void>;
}

const MAX_LOG_LINES = 500;

export const useServiceStore = create<ServiceState>((set, get) => ({
  services: [],
  logs: [],
  loading: false,
  error: null,

  fetchStatus: async () => {
    try {
      const statuses = await invoke<ServiceStatus[]>("get_service_status");
      set({ services: statuses });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startServices: async () => {
    set({ loading: true, error: null, logs: [] });
    try {
      const status = await invoke<ServiceStatus>("start_services");
      set({ services: [status], loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  stopServices: async () => {
    set({ loading: true, error: null });
    try {
      const status = await invoke<ServiceStatus>("stop_services");
      set({ services: [status], loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addLog: (line: string) => {
    set((state) => ({
      logs:
        state.logs.length >= MAX_LOG_LINES
          ? [...state.logs.slice(-MAX_LOG_LINES + 1), line]
          : [...state.logs, line],
    }));
  },

  clearLogs: () => set({ logs: [] }),
  clearError: () => set({ error: null }),

  initListeners: async () => {
    const unlisten1 = await listen<string>("log-line", (event) => {
      get().addLog(event.payload);
    });

    const unlisten2 = await listen<ServiceStatus[]>("service-status", (event) => {
      set({ services: event.payload });
    });

    const unlisten3 = await listen<string>("service-died", (event) => {
      get().addLog(`⚠️ Service "${event.payload}" stopped unexpectedly`);
      get().fetchStatus();
    });

    return () => {
      unlisten1();
      unlisten2();
      unlisten3();
    };
  },
}));
