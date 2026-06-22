import {
  Activity,
  AlertTriangle,
  Bell,
  CircleDot,
  Clock3,
  Cpu,
  Gauge,
  HardDrive,
  RadioTower,
  RefreshCcw,
  Search,
  Send,
  ShieldCheck,
  TerminalSquare,
  UploadCloud,
  WifiOff
} from "lucide-react";

type DeviceStatus = "online" | "degraded" | "offline" | "replaying";

type FleetDevice = {
  id: string;
  site: string;
  status: DeviceStatus;
  version: string;
  queueDepth: number;
  lastHeartbeat: string;
  telemetryRate: string;
  temperatureC: number;
  commandState: string;
  updateState: string;
};

type TelemetryEvent = {
  eventId: string;
  deviceId: string;
  type: string;
  receivedAt: string;
  payload: string;
};

const devices: FleetDevice[] = [
  {
    id: "edge-042",
    site: "Cold Room A",
    status: "online",
    version: "0.1.0",
    queueDepth: 0,
    lastHeartbeat: "12s ago",
    telemetryRate: "18/min",
    temperatureC: 4.1,
    commandState: "idle",
    updateState: "current"
  },
  {
    id: "edge-117",
    site: "Dock Sensor",
    status: "replaying",
    version: "0.1.0",
    queueDepth: 23,
    lastHeartbeat: "28s ago",
    telemetryRate: "41/min",
    temperatureC: 8.7,
    commandState: "ack pending",
    updateState: "current"
  },
  {
    id: "edge-205",
    site: "Freezer West",
    status: "degraded",
    version: "0.1.0",
    queueDepth: 7,
    lastHeartbeat: "2m ago",
    telemetryRate: "9/min",
    temperatureC: -17.8,
    commandState: "last succeeded",
    updateState: "canary eligible"
  },
  {
    id: "edge-319",
    site: "Yard Gateway",
    status: "offline",
    version: "0.0.9",
    queueDepth: 0,
    lastHeartbeat: "17m ago",
    telemetryRate: "0/min",
    temperatureC: 19.4,
    commandState: "unreachable",
    updateState: "behind"
  }
];

const telemetryEvents: TelemetryEvent[] = [
  {
    eventId: "01JZ9X6N9VD4",
    deviceId: "edge-042",
    type: "sensor.reading",
    receivedAt: "19:42:10",
    payload: "temp=4.1C humidity=61%"
  },
  {
    eventId: "01JZ9X6R7H2A",
    deviceId: "edge-117",
    type: "queue.replay",
    receivedAt: "19:42:06",
    payload: "batch=8 remaining=23"
  },
  {
    eventId: "01JZ9X6VQ5MK",
    deviceId: "edge-205",
    type: "health.check",
    receivedAt: "19:41:58",
    payload: "disk=72% cpu=68%"
  }
];

const statusStyles: Record<DeviceStatus, string> = {
  online: "bg-emerald-100 text-emerald-800 ring-emerald-200",
  degraded: "bg-amber-100 text-amber-900 ring-amber-200",
  offline: "bg-rose-100 text-rose-800 ring-rose-200",
  replaying: "bg-cyan-100 text-cyan-800 ring-cyan-200"
};

const statusIcons: Record<DeviceStatus, typeof CircleDot> = {
  online: CircleDot,
  degraded: AlertTriangle,
  offline: WifiOff,
  replaying: RefreshCcw
};

const apiUrl = import.meta.env.VITE_EDGEFLEET_API_URL ?? "http://localhost:8080";
const onlineDevices = devices.filter((device) => device.status === "online").length;
const queuedEvents = devices.reduce((total, device) => total + device.queueDepth, 0);

function App() {
  return (
    <main className="min-h-screen bg-zinc-100 text-zinc-950">
      <header className="border-b border-zinc-200 bg-white">
        <div className="mx-auto flex max-w-7xl flex-col gap-4 px-4 py-4 sm:px-6 lg:flex-row lg:items-center lg:justify-between lg:px-8">
          <div className="flex items-center gap-3">
            <div className="flex size-10 items-center justify-center rounded bg-zinc-950 text-white">
              <RadioTower aria-hidden="true" className="size-5" />
            </div>
            <div>
              <h1 className="text-xl font-semibold">EdgeFleet</h1>
              <p className="text-sm text-zinc-600">Fleet operations dashboard</p>
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <IconButton label="Refresh fleet data">
              <RefreshCcw aria-hidden="true" className="size-4" />
            </IconButton>
            <IconButton label="Open alerts">
              <Bell aria-hidden="true" className="size-4" />
            </IconButton>
            <button className="inline-flex h-9 items-center gap-2 rounded border border-zinc-300 bg-white px-3 text-sm font-medium shadow-panel transition hover:border-zinc-400">
              <Send aria-hidden="true" className="size-4" />
              Command
            </button>
          </div>
        </div>
      </header>

      <section className="border-b border-zinc-200 bg-zinc-50">
        <div className="mx-auto grid max-w-7xl gap-3 px-4 py-4 sm:grid-cols-2 sm:px-6 lg:grid-cols-4 lg:px-8">
          <Metric label="Online devices" value={`${onlineDevices}/${devices.length}`} icon={Activity} />
          <Metric label="Queued events" value={queuedEvents.toString()} icon={HardDrive} tone="cyan" />
          <Metric label="Canary-ready" value="1" icon={ShieldCheck} tone="emerald" />
          <Metric label="API target" value={apiUrl.replace(/^https?:\/\//, "")} icon={TerminalSquare} tone="zinc" />
        </div>
      </section>

      <div className="mx-auto grid max-w-7xl gap-4 px-4 py-5 sm:px-6 lg:grid-cols-[1fr_360px] lg:px-8">
        <section className="overflow-hidden rounded border border-zinc-200 bg-white shadow-panel">
          <div className="flex flex-col gap-3 border-b border-zinc-200 px-4 py-3 md:flex-row md:items-center md:justify-between">
            <div>
              <h2 className="text-base font-semibold">Fleet Inventory</h2>
              <p className="text-sm text-zinc-600">Heartbeat, queue, command, and rollout state</p>
            </div>
            <label className="relative block md:w-72">
              <span className="sr-only">Search devices</span>
              <Search aria-hidden="true" className="pointer-events-none absolute left-3 top-2.5 size-4 text-zinc-500" />
              <input
                className="h-9 w-full rounded border border-zinc-300 bg-white pl-9 pr-3 text-sm outline-none transition placeholder:text-zinc-500 focus:border-zinc-900"
                placeholder="Search devices"
                type="search"
              />
            </label>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full text-left text-sm">
              <thead className="bg-zinc-50 text-xs uppercase text-zinc-600">
                <tr>
                  <Th>Device</Th>
                  <Th>Status</Th>
                  <Th>Heartbeat</Th>
                  <Th>Queue</Th>
                  <Th>Telemetry</Th>
                  <Th>Command</Th>
                  <Th>OTA</Th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-200">
                {devices.map((device) => (
                  <DeviceRow key={device.id} device={device} />
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <aside className="space-y-4">
          <section className="rounded border border-zinc-200 bg-white shadow-panel">
            <div className="border-b border-zinc-200 px-4 py-3">
              <h2 className="text-base font-semibold">Live Telemetry</h2>
              <p className="text-sm text-zinc-600">Recent envelope activity</p>
            </div>
            <div className="divide-y divide-zinc-200">
              {telemetryEvents.map((event) => (
                <div key={event.eventId} className="px-4 py-3">
                  <div className="flex items-center justify-between gap-3">
                    <p className="font-mono text-xs text-zinc-500">{event.eventId}</p>
                    <p className="text-xs text-zinc-500">{event.receivedAt}</p>
                  </div>
                  <p className="mt-1 text-sm font-medium">{event.deviceId}</p>
                  <p className="text-sm text-zinc-600">{event.type}</p>
                  <p className="mt-2 rounded bg-zinc-100 px-2 py-1 font-mono text-xs text-zinc-700">
                    {event.payload}
                  </p>
                </div>
              ))}
            </div>
          </section>

          <section className="rounded border border-zinc-200 bg-white shadow-panel">
            <div className="border-b border-zinc-200 px-4 py-3">
              <h2 className="text-base font-semibold">System Health</h2>
              <p className="text-sm text-zinc-600">Phase 0 service shape</p>
            </div>
            <div className="grid grid-cols-2 gap-3 p-4">
              <HealthTile label="Control plane" value="scaffold" icon={Cpu} />
              <HealthTile label="NATS" value="planned" icon={RadioTower} />
              <HealthTile label="PostgreSQL" value="planned" icon={HardDrive} />
              <HealthTile label="Rollouts" value="modeled" icon={UploadCloud} />
            </div>
          </section>
        </aside>
      </div>
    </main>
  );
}

function DeviceRow({ device }: { device: FleetDevice }) {
  const StatusIcon = statusIcons[device.status];

  return (
    <tr className="align-top transition hover:bg-zinc-50">
      <Td>
        <div className="font-medium">{device.id}</div>
        <div className="text-xs text-zinc-500">{device.site}</div>
      </Td>
      <Td>
        <span className={`inline-flex items-center gap-1 rounded px-2 py-1 text-xs font-medium ring-1 ${statusStyles[device.status]}`}>
          <StatusIcon aria-hidden="true" className="size-3.5" />
          {device.status}
        </span>
      </Td>
      <Td>
        <div className="flex items-center gap-1.5">
          <Clock3 aria-hidden="true" className="size-4 text-zinc-500" />
          {device.lastHeartbeat}
        </div>
      </Td>
      <Td>{device.queueDepth}</Td>
      <Td>
        <div>{device.telemetryRate}</div>
        <div className="text-xs text-zinc-500">{device.temperatureC}C</div>
      </Td>
      <Td>{device.commandState}</Td>
      <Td>{device.updateState}</Td>
    </tr>
  );
}

function Metric({
  label,
  value,
  icon: Icon,
  tone = "rose"
}: {
  label: string;
  value: string;
  icon: typeof Activity;
  tone?: "rose" | "cyan" | "emerald" | "zinc";
}) {
  const toneClass = {
    rose: "bg-rose-100 text-rose-800",
    cyan: "bg-cyan-100 text-cyan-800",
    emerald: "bg-emerald-100 text-emerald-800",
    zinc: "bg-zinc-200 text-zinc-800"
  }[tone];

  return (
    <div className="flex min-h-24 items-center justify-between rounded border border-zinc-200 bg-white px-4 py-3 shadow-panel">
      <div>
        <p className="text-sm text-zinc-600">{label}</p>
        <p className="mt-1 text-2xl font-semibold">{value}</p>
      </div>
      <div className={`flex size-10 items-center justify-center rounded ${toneClass}`}>
        <Icon aria-hidden="true" className="size-5" />
      </div>
    </div>
  );
}

function HealthTile({
  label,
  value,
  icon: Icon
}: {
  label: string;
  value: string;
  icon: typeof Gauge;
}) {
  return (
    <div className="min-h-24 rounded border border-zinc-200 bg-zinc-50 p-3">
      <Icon aria-hidden="true" className="size-5 text-zinc-600" />
      <p className="mt-3 text-sm font-medium">{label}</p>
      <p className="text-xs text-zinc-500">{value}</p>
    </div>
  );
}

function IconButton({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <button
      aria-label={label}
      className="inline-flex size-9 items-center justify-center rounded border border-zinc-300 bg-white shadow-panel transition hover:border-zinc-400"
      title={label}
      type="button"
    >
      {children}
    </button>
  );
}

function Th({ children }: { children: React.ReactNode }) {
  return <th className="whitespace-nowrap px-4 py-3 font-semibold">{children}</th>;
}

function Td({ children }: { children: React.ReactNode }) {
  return <td className="whitespace-nowrap px-4 py-3">{children}</td>;
}

export default App;
