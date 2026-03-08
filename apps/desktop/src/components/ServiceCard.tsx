import { ServiceStatus } from "../stores/serviceStore";

interface ServiceCardProps {
  service: ServiceStatus;
  loading: boolean;
  onToggle: () => void;
  onOpenBrowser: () => void;
}

function formatUptime(seconds: number | null): string {
  if (seconds === null || seconds === undefined) return "--";
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h ${m}m`;
}

export function ServiceCard({
  service,
  loading,
  onToggle,
  onOpenBrowser,
}: ServiceCardProps) {
  const isRunning = service.running;

  return (
    <div className="glass-card-hover p-6 animate-fade-in">
      {/* Header Row */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div
            className={`w-10 h-10 rounded-xl flex items-center justify-center transition-all duration-500 ${
              isRunning
                ? "bg-emerald-500/15 text-emerald-400"
                : "bg-white/5 text-white/30"
            }`}
          >
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
              <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
            </svg>
          </div>
          <div>
            <h3 className="text-sm font-semibold text-white/90">
              {service.name}
            </h3>
            <p className="text-[11px] text-white/35 font-mono mt-0.5">
              {service.php_version
                ? `PHP ${service.php_version}`
                : "Version detecting..."}
            </p>
          </div>
        </div>

        {/* Toggle */}
        <button
          onClick={onToggle}
          disabled={loading}
          className={`relative w-12 h-7 rounded-full transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-accent-400/30 ${
            loading ? "opacity-50 cursor-wait" : "cursor-pointer"
          } ${
            isRunning
              ? "bg-emerald-500 shadow-[0_0_12px_rgba(52,211,153,0.3)]"
              : "bg-white/10"
          }`}
          aria-label={isRunning ? "Stop service" : "Start service"}
        >
          <div
            className={`absolute top-1 w-5 h-5 rounded-full bg-white shadow-md transition-all duration-300 ${
              isRunning ? "left-6" : "left-1"
            }`}
          />
        </button>
      </div>

      {/* Status Grid */}
      <div className="grid grid-cols-3 gap-3">
        <div className="bg-white/[0.03] rounded-xl px-3 py-2.5">
          <p className="text-[10px] text-white/30 uppercase tracking-wider font-medium mb-1">
            Status
          </p>
          <div className="flex items-center gap-1.5">
            <div
              className={
                isRunning ? "status-dot-running" : "status-dot-stopped"
              }
            />
            <span
              className={`text-xs font-medium ${
                isRunning ? "text-emerald-400" : "text-white/40"
              }`}
            >
              {isRunning ? "Running" : "Stopped"}
            </span>
          </div>
        </div>

        <div className="bg-white/[0.03] rounded-xl px-3 py-2.5">
          <p className="text-[10px] text-white/30 uppercase tracking-wider font-medium mb-1">
            Port
          </p>
          <span className="text-xs font-mono text-white/70">
            {service.port ?? "--"}
          </span>
        </div>

        <div className="bg-white/[0.03] rounded-xl px-3 py-2.5">
          <p className="text-[10px] text-white/30 uppercase tracking-wider font-medium mb-1">
            Uptime
          </p>
          <span className="text-xs font-mono text-white/70">
            {formatUptime(service.uptime_secs)}
          </span>
        </div>
      </div>

      {/* Open in Browser Button */}
      {isRunning && service.port && (
        <button
          onClick={onOpenBrowser}
          className="mt-4 w-full py-2.5 rounded-xl bg-accent-500/10 hover:bg-accent-500/20 text-accent-300 text-xs font-medium transition-all duration-200 flex items-center justify-center gap-2 group"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="group-hover:translate-x-0.5 transition-transform"
          >
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </svg>
          Open in Browser
        </button>
      )}
    </div>
  );
}
