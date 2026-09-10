use crate::metrics::BenchmarkSuite;

pub fn generate_report(suite: &BenchmarkSuite) -> String {
    let names: Vec<String> = suite
        .results
        .iter()
        .map(|r| format!("\"{}\"", r.name))
        .collect();

    let gate_reductions: Vec<String> = suite
        .results
        .iter()
        .map(|r| format!("{:.1}", r.gate_reduction_pct))
        .collect();

    let fidelities: Vec<String> = suite
        .results
        .iter()
        .map(|r| format!("{:.1}", r.fidelity_pct))
        .collect();

    let rows: String = suite.results.iter().map(|r| {
        let is_ok        = r.status == "ok";
        let status_badge = if is_ok { "badge-ok" } else { "badge-fail" };
        let status_text  = if is_ok { "SYS_OK" } else { "ERR_CRIT" };
        let out_gates    = r.input_gates
            - (r.input_gates as f64 * (r.gate_reduction_pct / 100.0)) as usize;

        format!(
            "<div class=\"telemetry-row\">\
              <div class=\"row-header\">\
                <span class=\"status-badge {}\">{}</span>\
                <span class=\"circuit-id\">{}</span>\
                <span class=\"row-latency\">{}ms</span>\
              </div>\
              <div class=\"matrix-subgrid\">\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Qubits</span><span class=\"m-cell-val\">{}</span></div>\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Gates I/O</span><span class=\"m-cell-val\">{}/{}</span></div>\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Depth I/O</span><span class=\"m-cell-val\">{}/{}</span></div>\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Gate Opt</span><span class=\"m-cell-val text-emerald\">{:.1}%</span></div>\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Depth Opt</span><span class=\"m-cell-val text-purple\">{:.1}%</span></div>\
                <div class=\"m-cell\"><span class=\"m-cell-lbl\">Fidelity</span><span class=\"m-cell-val text-ice\">{:.1}%</span></div>\
              </div>\
            </div>",
            status_badge, status_text, r.name, r.compile_ms,
            r.input_qubits, r.input_gates, out_gates,
            r.input_depth, r.output_depth,
            r.gate_reduction_pct, r.depth_reduction_pct, r.fidelity_pct
        )
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>QFORGE // QUANTUM INFRASTRUCTURE TELEMETRY</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/Chart.js/4.4.0/chart.umd.min.js"></script>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    background-color: #05070A;
    color: #94A3B8;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace;
    padding: 2rem;
    font-size: 12px;
    letter-spacing: -0.02em;
    height: 100vh;
    overflow: hidden;
  }}
  header {{
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #1E293B;
    padding-bottom: 1rem;
    margin-bottom: 1.5rem;
  }}
  h1 {{ color: #F1F5F9; font-size: 13px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.05em; }}
  h1 span {{ color: #475569; font-weight: 400; }}
  .engine-status {{ font-family: monospace; font-size: 11px; color: #475569; }}
  .terminal-workspace {{
    display: grid;
    grid-template-columns: 1fr 420px;
    gap: 1.5rem;
    height: calc(100vh - 100px);
  }}
  .analytics-left {{
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    min-width: 0;
  }}
  .kpi-matrix {{
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }}
  .kpi-card {{
    background: #090D14;
    border: 1px solid #1E293B;
    border-radius: 4px;
    padding: 1rem 1.25rem;
  }}
  .kpi-lbl {{ font-size: 9px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.05em; color: #64748B; }}
  .kpi-val {{ font-size: 24px; font-weight: 700; color: #F1F5F9; font-family: monospace; margin-top: 0.25rem; }}
  .chart-deck {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
    min-width: 0;
  }}
  .chart-panel {{
    background: #090D14;
    border: 1px solid #1E293B;
    border-radius: 4px;
    padding: 1.25rem;
    min-width: 0;
  }}
  .chart-title {{
    font-size: 10px;
    font-weight: 600;
    color: #64748B;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 1rem;
    border-left: 2px solid #334155;
    padding-left: 0.5rem;
  }}
  .canvas-viewport {{
    position: relative;
    height: 160px !important;
    width: 100% !important;
    overflow: hidden;
  }}
  .registry-stream-sidebar {{
    background: #090D14;
    border: 1px solid #1E293B;
    border-radius: 4px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }}
  .sidebar-banner {{
    padding: 0.85rem 1.25rem;
    border-bottom: 1px solid #1E293B;
    background: #0D131F;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }}
  .sidebar-title {{ font-size: 10px; font-weight: 700; color: #F1F5F9; text-transform: uppercase; letter-spacing: 0.05em; }}
  .sidebar-counter {{ font-size: 10px; font-family: monospace; color: #475569; }}
  .stream-scroll-container {{
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    overflow-y: scroll;
    flex-grow: 1;
  }}
  .telemetry-row {{
    background: #0B101A;
    border: 1px solid #1E293B;
    border-radius: 4px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }}
  .telemetry-row:hover {{ border-color: #475569; background: #0E1524; }}
  .row-header {{ display: flex; align-items: center; width: 100%; }}
  .status-badge {{
    font-size: 8px;
    font-weight: 700;
    font-family: monospace;
    padding: 0.15rem 0.35rem;
    border-radius: 2px;
    margin-right: 0.75rem;
    letter-spacing: 0.05em;
  }}
  .badge-ok   {{ background: rgba(0,230,118,0.1); color: #00E676; border: 1px solid rgba(0,230,118,0.2); }}
  .badge-fail {{ background: rgba(255,82,82,0.1);  color: #FF5252; border: 1px solid rgba(255,82,82,0.2); }}
  .circuit-id {{ font-weight: 600; font-family: monospace; color: #E2E8F0; flex-grow: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
  .row-latency {{ font-family: monospace; font-size: 10px; color: #475569; }}
  .matrix-subgrid {{
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.4rem;
    background: #06090F;
    padding: 0.5rem;
    border-radius: 2px;
    border: 1px solid rgba(255,255,255,0.01);
  }}
  .m-cell {{ display: flex; flex-direction: column; }}
  .m-cell-lbl {{ font-size: 8px; color: #475569; text-transform: uppercase; margin-bottom: 0.1rem; }}
  .m-cell-val {{ font-size: 11px; font-family: monospace; color: #94A3B8; }}
  .text-emerald {{ color: #00E676 !important; font-weight: 600; }}
  .text-purple  {{ color: #A5B4FC !important; font-weight: 600; }}
  .text-ice     {{ color: #38BDF8 !important; font-weight: 600; }}
</style>
</head>
<body>

<header>
  <div>
    <h1>QForge Engine Terminal <span>// Quantitative System Analytics</span></h1>
  </div>
  <div class="engine-status">
    SYS_REV: {version} &nbsp;&nbsp; TOTAL_LATENCY: {ms}ms &nbsp;&nbsp; FEED: ACTIVE
  </div>
</header>

<div class="terminal-workspace">
  <div class="analytics-left">
    <div class="kpi-matrix">
      <div class="kpi-card">
        <div class="kpi-lbl">Registry Records</div>
        <div class="kpi-val">{total}</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">Mean Optimization Delta</div>
        <div class="kpi-val text-emerald">{avg_gate:.1}%</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">Mean Fidelity Retention</div>
        <div class="kpi-val text-ice">{avg_fid:.1}%</div>
      </div>
      <div class="kpi-card">
        <div class="kpi-lbl">Pipeline Latency</div>
        <div class="kpi-val">{ms}ms</div>
      </div>
    </div>

    <div class="chart-deck">
      <div class="chart-panel">
        <div class="chart-title">Gate Structural Delta Profiles</div>
        <div class="canvas-viewport">
          <canvas id="gateChart"></canvas>
        </div>
      </div>
      <div class="chart-panel">
        <div class="chart-title">Fidelity Retainment Variances</div>
        <div class="canvas-viewport">
          <canvas id="fidelityChart"></canvas>
        </div>
      </div>
    </div>
  </div>

  <div class="registry-stream-sidebar">
    <div class="sidebar-banner">
      <span class="sidebar-title">Real-Time Core Registry Feed</span>
      <span class="sidebar-counter">TOTAL_UNITS: {total}</span>
    </div>
    <div class="stream-scroll-container">
      {rows}
    </div>
  </div>
</div>

<script>
const names = [{names}];
const gateReductions = [{gate_reductions}];
const fidelities = [{fidelities}];

const chartDefaults = {{
  responsive: true,
  maintainAspectRatio: false,
  plugins: {{ legend: {{ display: false }} }},
  scales: {{
    x: {{
      ticks: {{ color: '#475569', font: {{ family: 'monospace', size: 9 }} }},
      grid:  {{ display: false }}
    }},
    y: {{
      min: 0,
      max: 100,
      ticks: {{ color: '#475569', font: {{ family: 'monospace', size: 9 }}, stepSize: 50 }},
      grid:  {{ color: '#1E293B' }}
    }}
  }}
}};

new Chart(document.getElementById('gateChart'), {{
  type: 'line',
  data: {{
    labels: names,
    datasets: [{{
      data: gateReductions,
      borderColor: '#00E676',
      borderWidth: 1.2,
      pointRadius: 2,
      pointBackgroundColor: '#00E676',
      tension: 0.05,
      fill: false
    }}]
  }},
  options: chartDefaults
}});

new Chart(document.getElementById('fidelityChart'), {{
  type: 'line',
  data: {{
    labels: names,
    datasets: [{{
      data: fidelities,
      borderColor: '#38BDF8',
      borderWidth: 1.2,
      pointRadius: 2,
      pointBackgroundColor: '#38BDF8',
      tension: 0.05,
      fill: false
    }}]
  }},
  options: chartDefaults
}});
</script>
</body>
</html>"#,
        version = suite.qforge_version,
        total = suite.total_circuits,
        avg_gate = suite.avg_gate_reduction,
        avg_fid = suite.avg_fidelity,
        ms = suite.total_compile_ms,
        rows = rows,
        names = names.join(", "),
        gate_reductions = gate_reductions.join(", "),
        fidelities = fidelities.join(", "),
    )
}
