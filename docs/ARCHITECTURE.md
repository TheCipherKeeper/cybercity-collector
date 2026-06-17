# CyberCity Collector — Архитектура

## TL;DR

`cybercity-collector` — внешний **out-of-band** per-host наблюдатель за гостями
цифрового двойника CyberCity. Работает на хосте (гипервизор / K8s-нода), читает
гостей **снаружи** read-only, упаковывает наблюдения в подписанный (Ed25519)
конверт и шлёт в `cybercity-engine` по Kafka. Control-канал приходит от
`cybercity-manage`.

> Системный контекст (диаграмма, таблица ответственностей, доверительная
> граница, слои развёртывания) — в
> [`cybercity/ARCHITECTURE.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/ARCHITECTURE.md).
> Ниже — только внутреннее устройство коллектора. Канон состава и
> доверительной границы — в
> [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md).

## Текущее состояние vs цель (честно)

| | Сейчас | Цель |
|---|---|---|
| Размещение | In-guest MVP (эпоха `cybercity-agents`) | Out-of-band на гипервизоре/K8s-нодах |
| Зонды | Tail логов из `policy.host_permissions` | fs/net/mem/proc/syscall (ZFS-snapshot, eBPF, Volatility, Falco/Tetragon) |
| Подпись | Placeholder (`SecureTransport` — обёртка без крипто) | Настоящая Ed25519 (key id + nonce + timestamp, replay-protection) |
| Транспорт | `StdoutTransport` (печать в stdout) | Kafka/Redpanda (mTLS + ACL на продюсеров) |
| Envelope | Локальный `AgentEvent` | Канонический event envelope (см. `cybercity/CONVENTIONS.md`) |

Подробнее по статусу — в `README.md` и в
[`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md)
(раздел «Статус реализации»).

## Workspace из 6 crate'ов

```text
crates/
├── ccc-core       # runtime, config, lifecycle (self-health), policy, envelope-типы
├── ccc-host       # host-side read-only доступ (target: out-of-band зонды)
├── ccc-telemetry  # сбор событий из зондов
├── ccc-kafka      # transport (stub/Stdout) + подпись конверта (placeholder)
├── ccc-command    # control-команды от manage
└── ccc-node       # бинарник cybercity-collector (composition root)
```

### ccc-core

Владеет сквозными типами и конфигом:

- `Config` — TOML-конфиг с env-override через префикс `CCC_`
  (`CCC_NODE_ID`, `CCC_SERVICE_ID`, `CCC_KAFKA_BROKER`); валидация
  обязательных полей.
- `TelemetryConfig` — пути логов, `poll_interval_secs`, `buffer_size`.
- `Policy` / `HostPermission` — что коллектору разрешено читать на хосте и
  какие kinds команд исполнять.
- `Lifecycle` / `State` — self-health и состояние жизненного цикла;
  `record_tamper()` фиксирует попытки несанкционированного действия.
- `AgentEvent` — текущий (упрощённый) тип события; **цель** — канонический
  event envelope из `cybercity/CONVENTIONS.md`.

### ccc-host

`HostBridge` — единственная точка read-only доступа к хосту:

- `read_file` / `list_dir` — только в пределах `policy.host_permissions`.
- Защита от path traversal: `..` компоненты отклоняются, пути каноникализуются.
- **Цель:** заменить на настоящие out-of-band зонды (fs/net/mem/proc/syscall),
  работающие снаружи гостя. Сейчас — file-tail MVP.

### ccc-telemetry

`TelemetryCollector` — цикл сбора:

- Опрашивает `telemetry.log_paths` каждые `poll_interval_secs` через
  `HostBridge`, тейлит файлы, пакует строки в `AgentEvent` и шлёт в channel.
- **Цель:** источник событий — out-of-band зонды, а не in-guest логи.

### ccc-kafka

Transport-слой:

- `Transport`-трейт (`send_event` / `receive_command`) — абстракция над
  брокером, чтобы коллектор работал без реального Kafka.
- `StdoutTransport` — stub, печатает события в stdout (используется сейчас).
- `SecureTransport<T>` — обёртка-placeholder для envelope-шифрования, защиты от
  replay (nonce + timestamp) и подписи/верификации. **Реальная крипто — TODO.**
- `TopicNames` — `cc.events.<service_id>`, `cc.commands.<service_id>`,
  `cc.alerts`, `cc.audit`.
- Реальный Kafka за feature-флагом `real` (`rdkafka` + `rustls`) — закомментирован,
  не подключён.

### ccc-command

`CommandExecutor` — обработка control-команд от `cybercity-manage`:

- Проверяет `policy.allowed_command_kinds` (сейчас `status`, `read_file`).
- При несанкционированной команде пишет `record_tamper()` в lifecycle.
- Проверка подписи — placeholder (`InvalidSignature` возможен, но не проверяется
  крипто). **Реальная верификация — TODO.**

### ccc-node

Бинарник `cybercity-collector` (`src/main.rs`) — composition root:

- Грузит `Config`, строит `NodeIdentity`, `Lifecycle`, `HostBridge`, `TopicNames`.
- Три tokio-таски: telemetry-коллектор, transport (events → Kafka/stdout),
  command-listener (poll команд).
- Graceful shutdown по SIGINT/SIGTERM.

## Подписанный конверт (цель)

Каноническая форма события, пересекающего границы репозиториев — в
[`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md)
(раздел «Event envelope»). События коллектора дополнительно оборачиваются в
подписанный конверт:

- Ed25519-подпись с `key_id`, `nonce`, `timestamp` (replay-protection).
- `cybercity-engine` верифицирует подпись; Kafka — mTLS + ACL на продюсеров.
- Гости до брокера не достукиваются структурно (mgmt-сегмент без маршрута из
  range).

> **Сейчас конверт не реализован** — `SecureTransport` это placeholder.
> Реализация Ed25519 + реального Kafka-transport — отдельный заход (см.
> дорожную карту в `README.md`).

## История переименований

- Репо: `cybercity-agents` → **`cybercity-collector`** — переосмыслен из
  in-guest «агента» во внешний out-of-band per-host коллектор.
- Crate'ы: `ccna-*` → `ccc-*` (cyber city collector).
- Бинарник: `cybercity-node-agent` → `cybercity-collector`.
- Env-префикс: `CCNA_` → `CCC_`.

Полная история — в
[`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md)
(раздел «История переименований»).

## Модель безопасности (кратко)

- Коллектор живёт в mgmt-плоскости, **без маршрута из range** — недосягаем для
  цели. На его потоке считается scoring (trusted-плоскость; см.
  [`cybercity/adr/0002-trust-boundary.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0002-trust-boundary.md)).
- Read-only по отношению к гостям: никаких write/reset/изоляции напрямую —
  только наблюдение; действия — через `cybercity-manage`.
- Ключи Ed25519 — per-host, запечатанные (TPM/Secure Enclave где есть) — цель.
- In-guest обогащение — опционально, best-effort, **никогда** не источник для
  scoring.

## Связанные документы

- [`cybercity/ARCHITECTURE.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/ARCHITECTURE.md) — системная архитектура (контекст, ответственности, слои).
- [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md) — канон состава и доверительной границы.
- [`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md) — event envelope и кросс-репо конвенции.
- [`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md) — почему Rust и out-of-band.
- [`adr/0001-out-of-band-collector.md`](adr/0001-out-of-band-collector.md) — локальный ADR о переходе на out-of-band.
- [`DATA_FLOW.md`](DATA_FLOW.md) — поток данных коллектора.
- [`DEVELOPMENT.md`](DEVELOPMENT.md) — сборка, запуск, тестирование.