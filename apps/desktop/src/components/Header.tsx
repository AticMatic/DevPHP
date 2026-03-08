interface HeaderProps {
  version?: string;
}

export function Header({ version }: HeaderProps) {
  return (
    <header className="flex items-center justify-between px-6 py-4">
      <div className="flex items-center gap-3">
        {/* Logo */}
        <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-accent-400 to-accent-600 flex items-center justify-center shadow-lg glow-accent">
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white"
          >
            <polyline points="16 18 22 12 16 6" />
            <polyline points="8 6 2 12 8 18" />
          </svg>
        </div>
        <div>
          <h1 className="text-lg font-semibold tracking-tight">DevPHP</h1>
          <p className="text-[11px] text-white/40 font-medium tracking-wide uppercase">
            Development Environment
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        {version && (
          <span className="text-xs text-white/30 font-mono">v{version}</span>
        )}
        <div className="w-8 h-8 rounded-lg bg-white/5 flex items-center justify-center hover:bg-white/10 transition-colors cursor-pointer">
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white/40"
          >
            <circle cx="12" cy="12" r="3" />
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
          </svg>
        </div>
      </div>
    </header>
  );
}
