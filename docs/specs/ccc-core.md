# ccc-core

## Что это

Доменное ядро: конфигурация, политика доступа, lifecycle, типы событий.
Владеет сквозными типами, которые используют остальные crate'ы.

## Интерфейсы

- `Config` — TOML-конфиг с env-override через префикс `CCC_`
  (`CCC_NODE_ID`, `CCC_SERVICE_ID`, `CCC_KAFKA_BROKER`). Валидация обязательных
  полей (node_id, service_id).
- `TelemetryConfig` — пути логов, poll_interval_secs, buffer_size.
- `Policy` / `HostPermission` — что коллектору разрешено читать на хосте и
  какие kinds команд исполнять.
- `Lifecycle` / `State` — self-health и состояние жизненного цикла.
  `record_tamper()` фиксирует попытки несанкционированного действия.
  При 3+ tamper → State::Locked.
- `AgentEvent` — тип события (упрощённый). Цель — канонический event envelope.
- `NodeIdentity` — node_id, service_id, segment, boot_time.

## Типы

### Config

```rust
pub struct Config {
    pub node_id: String,       // обязательное
    pub service_id: String,   // обязательное
    pub segment: String,
    pub kafka_broker: String, // default: "localhost:9092"
    pub spool_path: String,    // default: "/var/lib/cybercity-agent/spool"
    pub telemetry: TelemetryConfig,
    pub policy: Policy,
}
```

### Policy

```rust
pub struct Policy {
    pub host_permissions: Vec<HostPermission>,
    pub allow_telemetry: bool,
    pub allowed_command_kinds: HashSet<String>,
}

pub enum HostPermission {
    ReadFile { paths: Vec<String> },
    ExecService { units: Vec<String> },
    WriteFile { paths: Vec<String> },
}
```

### Lifecycle / State

```rust
pub enum State {
    Initializing,
    Attesting,
    Active,
    Degraded,
    Locked,
}

pub struct Lifecycle {
    // watch::channel для подписки на смену состояния
    // tamper_count: Arc<AtomicUsize>
    // record_tamper() → count++, при >=3 → Locked
    // Active + tamper → Degraded
}
```

### AgentEvent

```rust
pub struct AgentEvent {
    pub ts: DateTime<Utc>,
    pub node_id: String,
    pub service_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
}
```

## Что есть

- Config: load (TOML + env), merge_env, validate.
- Policy: can_read_file, can_exec_service, can_run_command.
- Lifecycle: state machine, tamper counter, watch::channel.
- AgentEvent: new(), сериализация в JSON.
- NodeIdentity: from_config.

## Что TODO

- Канонический event envelope (event_id, parent_event_ids, correlation_id,
  tick, source_type, source_id, event_type, target_id, payload, status) —
  вместо упрощённого AgentEvent. См. `cybercity/CONVENTIONS.md`.
- Observation manifest: типы (RuntimeKind, ProbeConfig, ObservationManifest)
  + валидация. Manifest — декларативное описание за чем следить на цели:
  ```json
  {
    "manifest_version": 1,
    "target_id": "bank-web-01",
    "runtime_kind": "container",
    "probes": {
      "fs": { "watch_paths": ["/etc/nginx"], "ignore_patterns": ["*.log"] },
      "net": { "listen_ports": [80, 443], "suspicious_destinations": ["169.254.169.254"] },
      "proc": { "expected_processes": ["nginx"], "track_new_processes": true },
      "syscall": { "enabled": false },
      "lite": { "banner_check": "SSH-2.0", "heartbeat_interval_secs": 30 }
    }
  }
  ```
  Валидация: manifest_version (целое), target_id (непустая), runtime_kind
  (один из vm/container/lite), probes (объект, зависит от runtime_kind).
  Неизвестные поля игнорируются (forward compat). Пути абсолютные, без `..`.
- Config: поддержка observation manifest пути.

## Ограничения

- Config грузится из файла пути первого аргумента CLI
  (default: `config/example.toml`).
- Env-override: только CCC_NODE_ID, CCC_SERVICE_ID, CCC_KAFKA_BROKER.
- Policy.can_read_file использует starts_with — без каноникализации путей
  (каноникализация в ccc-host::normalize).

## Зависимости

- tokio, serde, toml, tracing, thiserror, chrono, serde_json