import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

/** Filet de sécurité : affiche toute erreur JS dans la page au lieu d'un écran blanc. */
function showFatal(title: string, detail: string) {
  const el = document.createElement("pre");
  el.textContent = `${title}\n\n${detail}`;
  el.style.cssText =
    "position:fixed;inset:0;z-index:9999;margin:0;padding:16px;overflow:auto;background:#fff0f0;color:#900;font:12px/1.5 monospace;white-space:pre-wrap";
  document.body.appendChild(el);
}
window.addEventListener("error", (e) =>
  showFatal("Erreur JavaScript", `${e.message}\n  à ${e.filename}:${e.lineno}:${e.colno}`),
);
window.addEventListener("unhandledrejection", (e) =>
  showFatal("Promesse rejetée (non catchée)", String((e as PromiseRejectionEvent).reason?.stack ?? (e as PromiseRejectionEvent).reason)),
);

class Boundary extends React.Component<
  { children: React.ReactNode },
  { err: Error | null }
> {
  state: { err: Error | null } = { err: null };
  static getDerivedStateFromError(err: Error) {
    return { err };
  }
  render() {
    if (this.state.err) {
      return <pre style={{ margin: 16, color: "#900", font: "12px/1.5 monospace", whiteSpace: "pre-wrap" }}>
        Crash React : {this.state.err.message}
        {"\n\n"}
        {this.state.err.stack}
      </pre>;
    }
    return this.props.children;
  }
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Boundary>
      <App />
    </Boundary>
  </React.StrictMode>,
);
