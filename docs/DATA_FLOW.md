# CyberCity Collector — Потоки данных

Этот документ описывает, как наблюдения движутся через `cybercity-collector`:
от зондов к подписанному конверту, через Kafka в `cybercity-engine`, и обратно
по control-каналу от `cybercity-manage`.

> Коллектор — часть **доверенной плоскости** (trusted): `cybercity-manage` +
> `cybercity-collector` + Kafka-брокер, живут в mgmt-сегменте **без маршрута из
> range**; на их потоке считается scoring. См.
> [`cybercity/adr/0002-trust-boundary.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0002-trust-boundary.md).

## Целевой поток (к чему идёт код)

```text
Хост (гипервизор / K8s-нода)
    │  out-of-band зонды (read-only, снаружи гостя)
    │   fs: ZFS-snapshot → ro-mount → integrity diff
    │   net: port-mirror / eBPF-XDP на tap → Zeek/Suricata
    │   mem: virsh dump-guest-memory → Volatility
    │   proc: /proc-walk, lsns
    │   syscall: Falco / Tetragon (контейнеры)
    ▼
ccc-host / ccc-telemetry
    │  наблюдение → AgentEvent (цель: канонический event envelope)
    ▼
ccc-kafka: SecureTransport
    │  обёртка в подписанный конверт: Ed25519 (key_id + nonce + timestamp)
    │  replay-protection; engine верифицирует подпись
    ▼
Kafka / Redpanda (mgmt-плоскость, mTLS + ACL на продюсеров)
    │  topic: cc.events.<service_id>
    │  гости до брокера не достукиваются структурно
    ▼
cybercity-engine
    │  авторитетный поток; на нём считается scoring
```

> Тот же probe-pipeline обслуживает все runtime-виды `{vm, container, lite}`
> (см. umbrella ADR-0004): vm/container — fs/net/mem/proc/syscall; `lite` —
> banner/socket-reachability + heartbeat. `honeypot` — purpose-флаг от manage,
> не отдельный поток. Класса «engine-synthesized service events» нет — движок
> регистратор, не симулятор.

### Control-канал (от manage к коллектору)

```text
cybercity-manage (контрольная плоскость)
    │  подписанная команда: «наблюдать X» / «снапшот сейчас» / «обновить политику»
    │  topic: cc.commands.<service_id>
    ▼
ccc-kafka: приём CommandEnvelope
    ▼
ccc-command: CommandExecutor
    │  проверка policy.allowed_command_kinds
    │  (цель) верификация подписи команды
    │  при несанкционированной команде → lifecycle.record_tamper()
    ▼
Действие (read_file / status / ...) — только в рамках политики
    │  reset / изоляция — НЕ через коллектор; через manage/фабрику
```

## Текущий поток (MVP, честно)

```text
Хост (in-guest, эпоха cybercity-agents)
    │  ccc-telemetry тейлит логи из telemetry.log_paths
    │  через ccc-host::HostBridge (read_file / list_dir, policy-ограничено)
    ▼
AgentEvent (упрощённый локальный тип)
    │  через mpsc::channel в ccc-node
    ▼
ccc-kafka: SecureTransport<StdoutTransport>
    │  подпись — placeholder (обёртка без крипто)
    │  transport — StdoutTransport: печать JSON в stdout
    ▼
(нет реального Kafka; нет engine-консьюмера)
```

```text
Control-канал (MVP)
    │  StdoutTransport::receive_command() всегда возвращает Ok(None)
    │  реальный poll команд от manage — TODO
```

## Чего пока нет (TODO)

- **Out-of-band зонды** (fs/net/mem/proc/syscall) — сейчас только tail логов.
- **Ed25519-подпись** конверта (`SecureTransport` — placeholder-обёртка).
- **Реальный Kafka-transport** (mTLS/SASL) — `StdoutTransport` вместо брокера.
  Закомментированная feature `real` (`rdkafka` + `rustls`) в `ccc-kafka`.
- **Канонический event envelope** из `cybercity/CONVENTIONS.md` — сейчас
  локальный `AgentEvent`; выравнивание с engine — часть out-of-band захода.
- **Верификация подписи** команд от manage — `InvalidSignature` возможен, но
  крипто не проверяется.

## Топики

`TopicNames::from_config` строит имена из `service_id`:

| Топик | Назначение |
|---|---|
| `cc.events.<service_id>` | события коллектора → engine |
| `cc.commands.<service_id>` | команды manage → коллектор |
| `cc.alerts` | алерты (цель) |
| `cc.audit` | аудит (цель) |

> Выравнивание с `city.*`-неймингом `cybercity-engine` — часть
> out-of-band рефакторинга.

## Схема события (цель)

Каноническая форма — в
[`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md)
(раздел «Event envelope»). Минимальный пример:

```json
{
  "event_id": "<uuid>",
  "parent_event_ids": ["<uuid>"],
  "correlation_id": "<scenario/incident>",
  "tick": 0,
  "timestamp": "<RFC3339>",
  "source_type": "collector",
  "source_id": "<node_id>",
  "event_type": "SCAN|ATTACK|...",
  "target_id": "<topology node id | null>",
  "payload": {},
  "status": "pending|processed|failed|suppressed"
}
```

Полный набор полей модели — в
[`cybercity-engine`/docs/MODELS.md](https://github.com/TheCipherKeeper/cybercity-engine/blob/main/docs/MODELS.md).

## Упорядочивание и durability (цель)

- События одного коллектора идут в порядке наблюдения (per-host FIFO).
- Replay-protection через `nonce` + `timestamp` в подписанном конверте.
- Оффлайн-буфер (`spool_path` в конфиге) — цель на случай недоступности брокера.

## Связанные документы

- [`docs/ARCHITECTURE.md`](ARCHITECTURE.md) — внутренняя архитектура.
- [`docs/DEVELOPMENT.md`](DEVELOPMENT.md) — сборка, запуск, тестирование.
- [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md) — канон состава и контрактов.
- [`cybercity/adr/0002-trust-boundary.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0002-trust-boundary.md) — доверительная граница.
- [`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md) — почему out-of-band на Rust.