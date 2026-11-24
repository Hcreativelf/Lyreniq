import { useCallback, useEffect, useState } from "react";
import { readRuntimeLabel } from "./services/desktop";

type Connection =
  | { state: "checking" }
  | { state: "connected"; label: string }
  | { state: "error" };

export function App() {
  const [connection, setConnection] = useState<Connection>({ state: "checking" });

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

  return (
    <main className="window-check">
      <div className="mark" aria-hidden="true">L</div>
      <p className="eyebrow">UNIVERSAL TUNER · 工程初始化</p>
      <h1>Lyreniq</h1>
      <p className="intro">Windows 桌面最小窗口</p>
      <section className="connection" aria-live="polite" aria-busy={connection.state === "checking"}>
        {connection.state === "checking" && <p>正在验证桌面连接…</p>}
        {connection.state === "connected" && (
          <>
            <p className="status">桌面连接已建立</p>
            <p className="runtime">{connection.label}</p>
          </>
        )}
        {connection.state === "error" && (
          <p>无法连接桌面后端。请通过项目根目录的 pnpm dev 启动原生窗口。</p>
        )}
      </section>
      <button type="button" onClick={() => void verifyConnection()} disabled={connection.state === "checking"}>
        重新验证桌面连接
      </button>
      <p className="scope">当前仅验证工程启动；调音与音频功能尚未实现。</p>
    </main>
  );
}
