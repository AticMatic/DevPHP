import { useEffect, useCallback } from "react";
import { useServiceStore, ServiceStatus } from "./stores/serviceStore";
import { Header } from "./components/Header";
import { ServiceCard } from "./components/ServiceCard";
import { LogViewer } from "./components/LogViewer";

const DEFAULT_SERVICE: ServiceStatus = {
  name: "PHP Development Server",
  running: false,
  port: null,
  pid: null,
  uptime_secs: null,
  php_version: null,
};

function App() {
  const {
    services,
    logs,
    loading,
    error,
    fetchStatus,
    startServices,
    stopServices,
    clearLogs,
    clearError,
    initListeners,
  } = useServiceStore();

  const phpService = services[0] || DEFAULT_SERVICE;

  // Initialize on mount
  useEffect(() => {
    fetchStatus();
    const cleanup = initListeners();
    return () => {
      cleanup.then((fn) => fn());
    };
  }, []);

  // Periodic status refresh
  useEffect(() => {
    const interval = setInterval(fetchStatus, 10000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const handleToggle = useCallback(() => {
    if (phpService.running) {
      stopServices();
    } else {
      startServices();
    }
  }, [phpService.running, startServices, stopServices]);

  const handleOpenBrowser = useCallback(() => {
    if (phpService.port) {
      // Use Tauri's opener plugin
      import("@tauri-apps/plugin-opener").then((opener) => {
        opener.openUrl(`http://localhost:${phpService.port}`);
      });
    }
  }, [phpService.port]);

  return (
    <div className="h-screen flex flex-col overflow-hidden bg-surface-400">
      <Header version="0.1.0" />

      <main className="flex-1 flex flex-col gap-4 px-6 pb-6 min-h-0 overflow-hidden">
        {/* Error Banner */}
        {error && (
          <div className="flex items-center gap-3 px-4 py-3 rounded-xl bg-red-500/10 border border-red-500/20 animate-fade-in">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              className="text-red-400 shrink-0"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="15" y1="9" x2="9" y2="15" />
              <line x1="9" y1="9" x2="15" y2="15" />
            </svg>
            <p className="text-xs text-red-300/90 flex-1">{error}</p>
            <button
              onClick={clearError}
              className="text-red-400/50 hover:text-red-400 transition-colors text-xs"
            >
              ✕
            </button>
          </div>
        )}

        {/* Service Card */}
        <ServiceCard
          service={phpService}
          loading={loading}
          onToggle={handleToggle}
          onOpenBrowser={handleOpenBrowser}
        />

        {/* Log Viewer */}
        <div className="flex-1 min-h-0">
          <LogViewer logs={logs} onClear={clearLogs} />
        </div>
      </main>

      {/* Bottom Bar */}
      <footer className="px-6 py-2.5 border-t border-white/[0.04] flex items-center justify-between">
        <span className="text-[10px] text-white/20 font-mono">
          ~/.devphp
        </span>
        <div className="flex items-center gap-1.5">
          <div
            className={`w-1.5 h-1.5 rounded-full ${
              phpService.running ? "bg-emerald-400" : "bg-white/15"
            }`}
          />
          <span className="text-[10px] text-white/25">
            {phpService.running ? "Services active" : "All stopped"}
          </span>
        </div>
      </footer>
    </div>
  );
}

export default App;
