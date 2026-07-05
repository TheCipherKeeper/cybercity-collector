# Backlog

Очередь задач. Агент берёт первый невыполненный пункт сверху вниз.
Человек управляет порядком и приоритетами.

## P1 — фундамент

- [x] [ccc-core] Убрать unused imports (HashMap, fmt, tracing) — blocking для verify.sh
- [ ] [ccc-core] Канонический event envelope → зависит от: ничего
- [ ] [ccc-core] Observation manifest: типы + валидация → зависит от: ничего
- [ ] [ccc-telemetry] Инкрементальный tail с offset → зависит от: ничего

## P2 — зависит от P1

- [ ] [ccc-telemetry] Чтение observation manifest → зависит от: manifest типы
- [ ] [ccc-command] Приём observation manifest → зависит от: manifest типы
- [ ] [ccc-kafka] Ed25519 подпись в SecureTransport → зависит от: event envelope
- [ ] [ccc-host] Probe trait → зависит от: manifest типы

## P3 — зависит от P2

- [ ] [ccc-telemetry] Зонды fs/net/proc/syscall/lite → зависит от: Probe trait, manifest
- [ ] [ccc-command] Верификация подписи команды (Ed25519) → зависит от: Ed25519
- [ ] [ccc-kafka] Real Kafka transport → зависит от: Ed25519
- [ ] [ccc-node] Загрузка observation manifest при старте → зависит от: чтение manifest

## P4 — финальная сборка

- [ ] [ccc-node] Корректный shutdown (drain channel, flush transport) → зависит от: ничего
- [ ] [ccc-node] SIGTERM обработка → зависит от: ничего
- [ ] [ccc-kafka] Оффлайн-буфер (spool_path) → зависит от: Real Kafka
- [ ] [ccc-node] Health endpoint → зависит от: ничего

## Правила

- Агент берёт первый невыполненный `[ ]` сверху вниз.
- Зависимости должны быть `[x]` перед началом задачи.
- После реализации: поставить `[x]`, перенести в спек (TODO → "Что есть").
- Человек меняет порядок, добавляет задачи, ставит приоритеты.