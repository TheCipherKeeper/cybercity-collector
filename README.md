# CyberCity — Agents

[![Part of CyberCity](https://img.shields.io/badge/CyberCity-composition-blueviolet)](#)
[![License: MIT](https://img.shields.io/badge/code-MIT-green)](LICENSE)
[![Docs: CC BY 4.0](https://img.shields.io/badge/docs-CC%20BY%204.0-lightgrey)](LICENSE-DOCS)

Коллекторы логов и телеметрии, работающие **внутри сегментов города**.
Шипят события в eventstore [`cybercity-core`](https://github.com/TheCipherKeeper/cybercity)
через gRPC/OTLP. Это blue-team глаза: они дают полигону наблюдаемость,
без них red team работает в тумане.

## Что собираем

- **Syslog / journald** — стандартные ОС-события с узлов города
- **DNS-логи** — все резолверы в каждом сегменте
- **HTTP access logs** — публичные порталы и API
- **OT-телеметрия** — SCADA-узлы (Modbus/TCP, DNP3, IEC-60870) — read-only
- **Network flow** — NetFlow/sFlow с edge-маршрутизаторов

## Принципы

- **Минимум зависимостей.** Один бинарь, статическая линковка.
- **Bounded buffers.** Агент не должен сам стать DoS-источником.
- **Local-first.** Если eventstore недоступен — складываем на диск, шлём позже.
- **No PII.** Полигон публичный, агенты не собирают реальные данные.

## Планируемые агенты

| Агент | Язык | Где живёт | Что шлёт |
|---|---|---|---|
| `journald-shipper` | Go | corp / ot / mgmt | syslog, journald |
| `dns-tap` | Go | mgmt | DNS-запросы |
| `http-access` | Go | public / intranet | access-логи |
| `ot-listener` | Go (C-binding для libmodbus) | ot | SCADA-телеметрия |
| `flow-exporter` | Rust | edge | NetFlow v9 / sFlow v5 |

## Композиция CyberCity

| Слой | Репозиторий |
|---|---|
| Профиль / витрина | [TheCipherKeeper](https://github.com/TheCipherKeeper/TheCipherKeeper) |
| Сайт | [thecipherkeeper.github.io](https://github.com/TheCipherKeeper/thecipherkeeper.github.io) |
| Core | [cybercity](https://github.com/TheCipherKeeper/cybercity) |
| Данные | [cybercity-data](https://github.com/TheCipherKeeper/cybercity-data) |
| Сценарии | [cybercity-scenarios](https://github.com/TheCipherKeeper/cybercity-scenarios) |
| UI | [cybercity-ui](https://github.com/TheCipherKeeper/cybercity-ui) |
| **Агенты (этот репо)** | **cybercity-agents** |
| Blueprints | [cybercity-blueprints](https://github.com/TheCipherKeeper/cybercity-blueprints) |

## Лицензия

- Код: [MIT](LICENSE)
- Документация: [CC BY 4.0](LICENSE-DOCS)
