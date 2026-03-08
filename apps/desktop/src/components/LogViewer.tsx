import { useEffect, useRef } from "react";

interface LogViewerProps {
  logs: string[];
  onClear: () => void;
}

export function LogViewer({ logs, onClear }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (containerRef.current && autoScrollRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [logs]);

  // Detect if user scrolled away from bottom
  const handleScroll = () => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    autoScrollRef.current = scrollHeight - scrollTop - clientHeight < 40;
  };

  return (
    <div className="glass-card flex flex-col h-full animate-slide-up">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-2">
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white/30"
          >
            <polyline points="4 17 10 11 4 5" />
            <line x1="12" y1="19" x2="20" y2="19" />
          </svg>
          <span className="text-xs font-medium text-white/50">Output</span>
          <span className="text-[10px] text-white/20 font-mono ml-1">
            {logs.length} lines
          </span>
        </div>
        <button
          onClick={onClear}
          className="text-[10px] text-white/25 hover:text-white/50 transition-colors uppercase tracking-wider font-medium"
        >
          Clear
        </button>
      </div>

      {/* Log Content */}
      <div
        ref={containerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto p-4 font-mono text-xs leading-relaxed min-h-0"
        style={{ minHeight: "200px", maxHeight: "400px" }}
      >
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-white/15 text-sm">
              Waiting for output...
            </p>
          </div>
        ) : (
          logs.map((line, i) => (
            <div
              key={i}
              className={`py-0.5 ${
                line.includes("error") || line.includes("Error")
                  ? "text-red-400/80"
                  : line.includes("⚠️") || line.includes("warning")
                  ? "text-amber-400/80"
                  : line.includes("Started") || line.includes("started")
                  ? "text-emerald-400/80"
                  : "text-white/50"
              }`}
            >
              <span className="text-white/15 mr-3 select-none">
                {String(i + 1).padStart(3, " ")}
              </span>
              {line}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
