# CyberCity — Collector

[![Part of CyberCity](https://img.shields.io/badge/CyberCity-composition-blueviolet)](https://github.com/TheCipherKeeper/cybercity)
[![License: MIT](https://img.shields.io/badge/code-MIT-green)](LICENSE)
[![Docs: CC BY 4.0](https://img.shields.io/badge/docs-CC%20BY%204.0-lightgrey)](LICENSE-DOCS)

Внешний **out-of-band коллектор** цифрового двойника CyberCity. Работает
**на хосте** (гипервизор / K8s-нода), **снаружи** гостевых VM/контейнеров,
read-only, в mgmt-плоскости — и потому **недосягаем** из range-сегмента.
Собирает телеметрию зондами (файлы, сеть, память, процессы, сисколлы),
подписывает события (Ed25519) и шлёт в `cybercity-engine` по Kafka как
**авторитетный** поток, на котором считается scoring. Control-канал — от
`cybercity-manage`.

> Канон композиции, контрактов и доверительной границы —
> [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md).

## Почему снаружи, а не в госте

В кибер-полигоне среда враждебна: задача красных — ослепить защиту. Любая
телеметрия *внутри* гостя compromisable по определению. Поэтому доверенная
телеметрия для scoring собирается **снаружи** — на привилегию выше цели:
гость структурно не может до неё достучаться (нет маршрута в mgmt-плоскость,
отдельное ядро для VM / gVisor·Kata для контейнеров). In-guest обогащение,
если нужно — опционально и best-effort, **никогда** не источник для scoring.

## Зонды (целевые)

Зонд — pluggable модуль; набор зависит от runtime цели.

| Зонд | VM (Proxmox/ZFS) | Контейнер (K8s) |
|---|---|---|
| fs | ZFS-snapshot → ro-mount → integrity diff | overlay2-walk |
| net | port-mirror / eBPF-XDP на tap → Zeek/Suricata | mirror бриджа + conntrack |
| mem | `virsh dump-guest-memory` → Volatility | `/proc/<pid>/mem`, `criu` |
| proc | (через mem-dump) | `/proc`-walk, `lsns` |
| syscall | — | Falco / Tetragon |

## Архитектура (код)

```
crates/
├── ccc-core       # runtime, config, lifecycle (self-health), policy, envelope
├── ccc-host       # host-side доступ (read-only зонды)
├── ccc-telemetry  # сбор событий из зондов
├── ccc-kafka      # Kafka/Redpanda transport + подпись конверта (Ed25519)
├── ccc-command    # control-команды от manage
└── ccc-node       # бинарник cybercity-collector
```

## Статус

**Каркас в переходе.** Текущий код — ещё in-guest MVP из эпохи `cybercity-agents`
(host-bridge / tail логов / stub-transport / placeholder-подпись). Рефакторинг
зондов в out-of-band (fs/net/mem/proc/syscall), настоящая Ed25519-подпись
событий и реальный Kafka-transport (mTLS) — в дорожной карте. Имена уже
приведены к коллектору: crate'ы `ccna-*` → `ccc-*` (cyber city collector),
бинарник `cybercity-node-agent` → `cybercity-collector`, env-префикс `CCNA_` → `CCC_`.

## Сборка

```bash
rustc --version   # stable
cargo build
cargo build --release
```

## Запуск

```bash
cp config/example.toml config/local.toml
cargo run --bin cybercity-collector -- config/local.toml
```

В текущем MVP коллектор читает логи из разрешённых `policy.host_permissions`
и печатает события в stdout (вместо настоящего Kafka — `StdoutTransport`).

## Принципы

- **Out-of-band.** Наблюдатель живёт на привилегию выше цели и недосягаем из
  range-сегмента — tamper-proof по конструкции, а не по усилию.
- **Read-only.** Коллектор не пишет в гости; действия (reset/изоляция) —
  через `cybercity-manage`/фабрику, не через in-guest код.
- **Signed events.** Каждый конверт подписан (Ed25519, key id + nonce +
  timestamp); `engine` верифицирует. Kafka — mTLS + ACL на продюсеров.
- **Best-effort in-guest — never trusted.** Опциональное обогащение внутри
  гостя не используется для scoring; «агент замолчал» — сам по себе сигнал.

## Дорожная карта

1. Зонды out-of-band: fs (ZFS-snapshot) → net (mirror/Zeek) → mem (Volatility);
   для контейнеров — Falco/Tetragon.
2. Настоящая Ed25519-подпись событий (key id + nonce + timestamp,
   replay-protection); `engine` верифицирует.
3. Реальный Kafka-transport (mTLS/SASL) вместо `StdoutTransport`/mock.
4. Per-host теги (`host_id`, `target_id`, `source`, `tick`) для корреляции в engine.
5. Control-канал от `cybercity-manage` (наблюдать цель / снапшот сейчас / политика).
6. Ключи per-host, запечатанные (TPM/Secure Enclave где есть).

## Лицензия

- Код: [MIT](LICENSE)
- Документация: [CC BY 4.0](LICENSE-DOCS)