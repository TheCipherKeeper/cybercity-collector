# ccc-telemetry

## Что это

Оркестрация сбора телеметрии. Циклично опрашивает зонды, нормализует
наблюдения в события, мультиплексирует источники, отправляет в channel.

## Интерфейсы

- `TelemetryCollector::new(config, bridge, sender)` — создание.
- `TelemetryCollector::run(node_id, service_id)` — бесконечный цикл сбора.

## Типы

```rust
pub struct TelemetryCollector {
    config: TelemetryConfig,
    bridge: HostBridge,
    sender: mpsc::Sender<AgentEvent>,
}

pub struct LogLine {
    pub source: PathBuf,
    pub line_no: usize,
    pub text: String,
    pub ts: DateTime<Utc>,
}

pub enum TelemetryError {
    Host(ccc_host::HostError),
    ChannelClosed,
}
```

## Что есть

- Цикл сбора: interval (poll_interval_secs) → collect_pattern для каждого
  пути из config.log_paths.
- collect_pattern: если файл — tail_file, если директория — list_dir +
  tail_file для каждого файла (не hidden).
- tail_file: read_file → строки → LogLine → AgentEvent("log_line") → sender.
- Skip для hidden файлов (начинаются с `.`).
- MissedTickBehavior::Skip — пропуск пропущенных тиков.

## Что TODO

- Чтение observation manifest: активация зондов по конфигу manifest,
  а не только log_paths.
- Probe-matrix: fs/net/proc/syscall/lite зонды в зависимости от runtime_kind.
- Канонический event envelope вместо AgentEvent("log_line").
- Инкрементальный tail: сохранение offset, чтение только новых строк
  (сейчас перечитывает весь файл каждый тик).
- Мультиплексирование: параллельный опрос нескольких зондов.
- Backpressure: обработка переполнения channel (сейчас send().await блокирует).

## Ограничения

- tail_file читает ВЕСЬ файл каждый тик — не подходит для больших логов.
- Нет сохранения состояния между тиками (offset, last_read).
- Источник данных — только файлы из telemetry.log_paths.
- Только один тип события: "log_line".

## Зависимости

- ccc-core, ccc-host, tokio, chrono, serde_json, thiserror, tracing