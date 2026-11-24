import { ArrowDown, ArrowUp, Check, FlaskConical, MicOff, RefreshCw } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { scenarioLabels, tunerCopy as t } from "../content/tuner.zh-CN";
import {
  createUnavailableBackend,
  formatCents,
  pointerCents,
  type TunerState,
  tuningDirection,
  validateTunerState,
} from "../services/audio-backend";
import { createDemoBackend, type DemoBackend, type DemoScenario } from "../services/demo-backend";
import { Badge, Button } from "./ui";

const unavailable = createUnavailableBackend();
const majorTicks = [-50, -25, 0, 25, 50];
const minorTicks = [-40, -30, -20, -10, 10, 20, 30, 40];
const tickX = (value: number) => 300 + value * 5.2;

export function TunerWorkspace({
  demoFactory = createDemoBackend,
}: {
  demoFactory?: () => DemoBackend;
}) {
  const [demo, setDemo] = useState<DemoBackend | null>(null);
  const [state, setState] = useState<TunerState>({ status: "idle" });
  const [scenario, setScenario] = useState<DemoScenario>("tuned");
  const scenarioControl = useRef<HTMLSelectElement>(null);
  const entryControl = useRef<HTMLButtonElement>(null);
  const wasDemo = useRef(false);
  const backend = demo ?? unavailable;
  useEffect(() => {
    if (demo) scenarioControl.current?.focus();
    else if (wasDemo.current) entryControl.current?.focus();
    wasDemo.current = demo !== null;
  }, [demo]);
  useEffect(() => {
    let active = true;
    const unsubscribe = backend.subscribe((next) => {
      if (active) setState(validateTunerState(next));
    });
    if (backend.source === "demo")
      void backend.start().catch(() => {
        if (active) setState({ status: "error", code: "input-unavailable" });
      });
    return () => {
      active = false;
      unsubscribe();
      void backend.stop().catch(() => {});
    };
  }, [backend]);

  function enterDemo() {
    setState({ status: "idle" });
    setScenario("tuned");
    setDemo(demoFactory());
  }
  function exitDemo() {
    setState({ status: "idle" });
    setDemo(null);
    setScenario("tuned");
  }
  function selectScenario(next: DemoScenario) {
    setScenario(next);
    demo?.selectScenario(next);
  }

  const reading = state.status === "detected" ? state.reading : null;
  const signal = reading ?? ("signal" in state ? state.signal : null);
  const direction = reading ? tuningDirection(reading.cents) : null;
  const DirectionIcon = direction === "flat" ? ArrowUp : direction === "sharp" ? ArrowDown : Check;
  const overflow = reading !== null && Math.abs(reading.cents) > 50;
  const statusText = direction
    ? t.directions[direction]
    : state.status !== "detected"
      ? t.state[state.status]
      : "";
  const help =
    state.status === "error" && state.code === "invalid-frame"
      ? t.help.invalid
      : state.status !== "detected"
        ? t.help[state.status]
        : t.tolerance;
  const pointerLabel = reading
    ? `${t.deviation} ${formatCents(reading.cents)} cents${overflow ? `，${t.overflow}` : ""}`
    : `${t.deviation}：暂无读数`;

  return (
    <div
      className="tuner-workspace"
      data-mode={demo ? "demo" : "default"}
      data-state={state.status}
    >
      <div className={`tuner-banner${demo ? " is-demo" : ""}`}>
        <div className="tuner-provenance">
          {demo ? (
            <FlaskConical size={16} aria-hidden="true" />
          ) : (
            <MicOff size={16} aria-hidden="true" />
          )}
          <div>
            {demo && <strong>{t.demo}</strong>}
            <p id="feature-availability">{demo ? t.demoHint : t.idleHint}</p>
          </div>
        </div>
        {demo ? (
          <div className="demo-controls">
            <label className="sr-only" htmlFor="demo-scenario">
              {t.scenario}
            </label>
            <select
              ref={scenarioControl}
              id="demo-scenario"
              value={scenario}
              onChange={(event) => selectScenario(event.target.value as DemoScenario)}
            >
              {(Object.keys(scenarioLabels) as DemoScenario[]).map((id) => (
                <option key={id} value={id}>
                  {scenarioLabels[id]}
                </option>
              ))}
            </select>
            <Button tone="quiet" onClick={exitDemo}>
              {t.exitDemo}
            </Button>
          </div>
        ) : (
          <Button ref={entryControl} onClick={enterDemo}>
            <FlaskConical size={14} aria-hidden="true" />
            {t.enterDemo}
          </Button>
        )}
      </div>

      <section
        className={`tuner-panel${state.status === "error" ? " has-error" : ""}`}
        aria-label={t.title}
      >
        <div className="tuner-meta">
          <span>
            {t.target}{" "}
            <b>
              {reading
                ? `${reading.noteName}${reading.octave} · ${reading.targetHz.toFixed(2)} Hz`
                : "—"}
            </b>
          </span>
          <span>
            {t.reference} <b>440 Hz</b>
          </span>
        </div>
        <div className="pitch-readout">
          <div className="frequency-readout">
            <span className="readout-label">{t.frequency}</span>
            <output aria-label={t.frequency} aria-live="off" data-testid="measured-frequency">
              {reading ? reading.frequencyHz.toFixed(2) : "—"}
              <small> Hz</small>
            </output>
          </div>
          <div className="note-readout">
            <span className="readout-label">{t.note}</span>
            <output aria-label={t.note} aria-live="off">
              {reading ? (
                <>
                  {reading.noteName}
                  <span className="note-octave">{reading.octave}</span>
                </>
              ) : (
                "—"
              )}
            </output>
          </div>
          <div className="cents-readout">
            <span className="readout-label">{t.deviation}</span>
            <output aria-label={t.deviation} aria-live="off">
              {reading ? formatCents(reading.cents) : "—"}
              <small> cents</small>
            </output>
          </div>
        </div>
        <svg className="tuning-gauge" viewBox="0 0 600 70" role="img" aria-label={pointerLabel}>
          <title>{pointerLabel}</title>
          <rect className="gauge-center" x="284.4" y="11" width="31.2" height="29" rx="3" />
          <line className="gauge-axis" x1="40" x2="560" y1="27" y2="27" />
          {minorTicks.map((value) => (
            <line
              key={value}
              className="gauge-tick"
              x1={tickX(value)}
              x2={tickX(value)}
              y1="23"
              y2="31"
            />
          ))}
          {majorTicks.map((value) => (
            <g key={value}>
              <line
                className="gauge-tick major"
                x1={tickX(value)}
                x2={tickX(value)}
                y1="18"
                y2="36"
              />
              <text className="gauge-label" x={tickX(value)} y="57" textAnchor="middle">
                {value > 0 ? `+${value}` : value}
              </text>
            </g>
          ))}
          {reading && (
            <g
              className="gauge-needle"
              data-testid="tuning-needle"
              data-cents={pointerCents(reading.cents)}
              transform={`translate(${tickX(pointerCents(reading.cents))} 0)`}
            >
              <path d="M-5 5 L5 5 L0 13 Z" />
              <line x1="0" x2="0" y1="12" y2="39" />
            </g>
          )}
        </svg>
        <div className={`signal-state ${direction ?? state.status}`}>
          <p role="status">
            {direction && <DirectionIcon size={15} aria-hidden="true" />}
            {statusText}
          </p>
          <span>{overflow ? t.overflow : help}</span>
          {state.status === "error" && demo && (
            <Button className="retry-demo" tone="quiet" onClick={enterDemo}>
              <RefreshCw size={12} aria-hidden="true" />
              {t.retry}
            </Button>
          )}
        </div>
        <div className="signal-metrics">
          <div>
            <div className="metric-caption">
              <span>{t.level}</span>
              <output aria-label={t.level} aria-live="off">
                {signal ? `${signal.rmsDbfs.toFixed(1)} dBFS` : "—"}
              </output>
            </div>
            {signal ? (
              <meter
                className="tuner-meter"
                min={-90}
                max={0}
                value={signal.rmsDbfs}
                aria-label={t.level}
              />
            ) : (
              <span className="empty-meter" />
            )}
          </div>
          <div>
            <div className="metric-caption">
              <span title={t.metricsHint}>{t.confidence}</span>
              <output aria-label={t.confidence} aria-live="off">
                {signal ? `${Math.round(signal.confidence * 100)}%` : "—"}
              </output>
            </div>
            {signal ? (
              <meter
                className="tuner-meter"
                min={0}
                max={1}
                value={signal.confidence}
                aria-label={t.confidence}
              />
            ) : (
              <span className="empty-meter" />
            )}
          </div>
          <Badge>{demo ? "Demo" : t.unavailable}</Badge>
        </div>
      </section>
    </div>
  );
}
