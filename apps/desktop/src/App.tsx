import {
  AudioLines,
  ChevronRight,
  FileAudio,
  Headphones,
  Moon,
  Music2,
  RefreshCw,
  Settings2,
  ShieldCheck,
  Sun,
} from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { Badge, Button } from "./components/ui";
import { copy, type PageId, pages, type Theme } from "./content/zh-CN";
import { readRuntimeLabel } from "./services/desktop";

type Connection =
  | { state: "checking" }
  | { state: "connected"; label: string }
  | { state: "error" };
const icons = { tuner: AudioLines, files: FileAudio, tone: Music2, settings: Settings2 };

export function App() {
  const [page, setPage] = useState<PageId>("tuner");
  const [theme, setTheme] = useState<Theme>("dark");
  const [connection, setConnection] = useState<Connection>({ state: "checking" });
  const heading = useRef<HTMLHeadingElement>(null);
  const previousPage = useRef(page);
  const current = pages[page];
  const Icon = icons[page];
  const verifyConnection = useCallback(async () => {
    setConnection({ state: "checking" });
    try {
      setConnection({ state: "connected", label: await readRuntimeLabel() });
    } catch {
      setConnection({ state: "error" });
    }
  }, []);
  useEffect(() => {
    void verifyConnection();
  }, [verifyConnection]);
  useEffect(() => {
    if (previousPage.current !== page) {
      heading.current?.focus();
      previousPage.current = page;
    }
  }, [page]);

  return (
    <div className="app-shell" data-theme={theme}>
      <a className="skip-link" href="#main-content">
        {copy.skip}
      </a>
      <aside className="sidebar" aria-label={copy.sidebar}>
        <div className="brand">
          <svg className="brand-mark" viewBox="0 0 36 36" fill="none" aria-hidden="true">
            <path d="M10 8v11a8 8 0 0 0 16 0V8M18 8v22M14 12v8m8-8v8" />
          </svg>
          <div>
            <span className="brand-name">Lyreniq</span>
            <span className="brand-caption">UNIVERSAL TUNER</span>
          </div>
        </div>
        <p className="nav-caption">{copy.workspace}</p>
        <nav className="navigation" aria-label={copy.navigation}>
          {(Object.keys(pages) as PageId[]).map((id, index) => {
            const NavIcon = icons[id];
            return (
              <button
                key={id}
                type="button"
                className={id === "settings" ? "nav-item nav-settings" : "nav-item"}
                aria-current={page === id ? "page" : undefined}
                onClick={() => setPage(id)}
              >
                <NavIcon size={19} aria-hidden="true" />
                <span>{pages[id].label}</span>
                <span className="nav-number" aria-hidden="true">
                  0{index + 1}
                </span>
              </button>
            );
          })}
        </nav>
        <div className="sidebar-bottom">
          <Button
            className="theme-toggle"
            tone="quiet"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            aria-label={theme === "dark" ? copy.toLight : copy.toDark}
          >
            {theme === "dark" ? (
              <Sun size={17} aria-hidden="true" />
            ) : (
              <Moon size={17} aria-hidden="true" />
            )}
            <span>{theme === "dark" ? copy.lightMode : copy.darkMode}</span>
            <span className="theme-swatch" aria-hidden="true" />
          </Button>
          <div className="sidebar-note">
            <ShieldCheck size={14} aria-hidden="true" />
            {copy.local}
          </div>
        </div>
      </aside>
      <header className="topbar">
        <div className="breadcrumb">
          <span>{copy.workspace}</span>
          <ChevronRight size={14} aria-hidden="true" />
          <span>{current.label}</span>
        </div>
        <div className="topbar-meta">
          <span className="idle-dot" aria-hidden="true" />
          {copy.notListening}
          <span className="platform">Windows</span>
        </div>
      </header>
      <main id="main-content" className="workspace" tabIndex={-1}>
        <div className="page-heading">
          <div>
            <p className="eyebrow">{current.eyebrow}</p>
            <h1 ref={heading} tabIndex={-1}>
              {current.label}
            </h1>
            <p className="page-description">{current.description}</p>
          </div>
          {page !== "settings" && (
            <Button tone="primary" disabled aria-describedby="feature-availability">
              <Icon size={16} aria-hidden="true" />
              {current.action}
            </Button>
          )}
        </div>
        {page === "settings" ? (
          <section className="settings-surface" aria-labelledby="appearance-title">
            <div className="section-heading">
              <div>
                <h2 id="appearance-title">{copy.appearance}</h2>
                <p>{copy.appearanceDescription}</p>
              </div>
              <Badge>{copy.currentSession}</Badge>
            </div>
            <fieldset className="theme-options" aria-labelledby="appearance-title">
              {(["dark", "light"] as Theme[]).map((option) => (
                <button
                  type="button"
                  key={option}
                  className="theme-option"
                  aria-pressed={theme === option}
                  onClick={() => setTheme(option)}
                >
                  <span
                    className={
                      option === "dark"
                        ? "theme-preview preview-dark"
                        : "theme-preview preview-light"
                    }
                    aria-hidden="true"
                  >
                    <span />
                    <span>
                      <i />
                      <i />
                      <i />
                    </span>
                  </span>
                  <span className="theme-option-label">
                    {option === "dark" ? (
                      <Moon size={16} aria-hidden="true" />
                    ) : (
                      <Sun size={16} aria-hidden="true" />
                    )}
                    {option === "dark" ? copy.darkTheme : copy.lightTheme}
                    <span className="selection-dot" aria-hidden="true" />
                  </span>
                </button>
              ))}
            </fieldset>
            <div className="settings-placeholder">
              <div>
                <h2>{copy.audioSettings}</h2>
                <p>{copy.audioSettingsDescription}</p>
              </div>
              <Badge>{copy.unavailable}</Badge>
            </div>
          </section>
        ) : (
          <section className="workspace-surface" aria-labelledby="empty-title">
            <div className="surface-label">
              <span className="section-kicker">{current.panelLabel}</span>
              <Badge>{copy.unavailable}</Badge>
            </div>
            <div className="empty-state">
              <div className="empty-symbol" aria-hidden="true">
                <span />
                <Icon size={34} strokeWidth={1.4} />
                <span />
              </div>
              <p className="empty-overline">{current.emptyOverline}</p>
              <h2 id="empty-title">{current.emptyTitle}</h2>
              <p id="feature-availability" className="empty-description">
                {current.emptyDescription}
              </p>
              <div className="empty-detail">
                <Headphones size={15} aria-hidden="true" />
                {current.hint}
              </div>
            </div>
            <ul className="workspace-outline" aria-label={copy.plannedTools}>
              {current.tools.map((tool) => (
                <li key={tool}>
                  <span className="tool-dot" aria-hidden="true" />
                  <span>{tool}</span>
                  <span className="tool-state">{copy.pending}</span>
                </li>
              ))}
            </ul>
          </section>
        )}
        <p className="privacy-note">
          <ShieldCheck size={15} aria-hidden="true" />
          {copy.privacy}
        </p>
      </main>
      <footer className="statusbar">
        <section
          className={`connection connection-${connection.state}`}
          aria-live="polite"
          aria-busy={connection.state === "checking"}
          aria-label={copy.connection}
        >
          <span className="connection-dot" aria-hidden="true" />
          <span>
            {connection.state === "checking"
              ? copy.checking
              : connection.state === "connected"
                ? copy.connected
                : copy.connectionError}
          </span>
          {connection.state === "connected" && <span className="runtime">{connection.label}</span>}
        </section>
        <Button
          className="reconnect"
          tone="quiet"
          onClick={() => void verifyConnection()}
          disabled={connection.state === "checking"}
          aria-label={copy.reconnect}
          title={copy.reconnect}
        >
          <RefreshCw size={13} aria-hidden="true" />
        </Button>
        <span className="statusbar-note">{copy.localOnly}</span>
      </footer>
    </div>
  );
}
