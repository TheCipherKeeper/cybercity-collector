# ccc-node

## Что это

Composition root: бинарник `cybercity-collector`. Композитит все слои, запускает
3 tokio-таски, обрабатывает graceful shutdown.

## Интерфейсы

- `main()` — точка входа, бинарник `cybercity-collector`.
- CLI: первый аргумент — путь к конфигу (default: `config/example.toml`).

## Что есть

- Загрузка Config из пути первого аргумента CLI.
- Создание: NodeIdentity, Lifecycle, Policy (Arc), HostBridge (Arc), TopicNames.
- Lifecycle: Initializing → Active при старте.
- mpsc::channel для событий (buffer_size из конфига).
- 3 tokio-таски:
  1. **Telemetry collector** — TelemetryCollector::run, шлёт события в channel.
  2. **Transport** — читает события из channel, шлёт через SecureTransport
     (StdoutTransport) в topic cc.events.\<service_id\>.
  3. **Command listener** — poll StdoutTransport::receive_command каждые 10с,
     исполняет через CommandExecutor.
- Graceful shutdown по SIGINT (ctrl_c).
- При shutdown: lifecycle → Initializing, abort всех тасок.

## Что TODO

- SIGTERM обработка (сейчас только ctrl_c).
- Корректный shutdown: завершение telemetry → drain channel → flush transport
  (сейчас просто abort).
- Command listener с реальным transport (сейчас poll StdoutTransport всегда
  возвращает None).
- Загрузка observation manifest при старте.
- Lifecycle: переход через Attesting перед Active (attestation — цель).
- Health endpoint: HTTP/metrics для внешнего мониторинга.
- spool_path: буферизация событий при недоступности брокера.

## Ограничения

- Shutdown: abort тасок, не graceful drain (события в channel теряются).
- Command listener: poll каждые 10с на stub transport — нет реальных команд.
- Lifecycle при shutdown → Initializing (transitional), не отдельный
  ShuttingDown state.

## Зависимости

- ccc-core, ccc-host, ccc-telemetry, ccc-kafka, ccc-command
- tokio, tracing, tracing-subscriber, anyhow