# ADR-0001: Out-of-band коллектор, не in-guest агент

## Status

Accepted

## Context

CyberCity — кибер-полигон: задача красных — ослепить защиту. Любая телеметрия
*внутри* гостя compromisable по определению: in-guest «агент» живёт в
скомпрометированной среде, может быть подменён, отключён или заставлен врать;
тянуть брокера из гостя нельзя. Если считать scoring на потоке из гостей —
атакующий подделывает свой результат.

Текущий код коллектора — наследие in-guest MVP эпохи `cybercity-agents`
(`ccna-*` crate'ы, бинарник `cybercity-node-agent`): tail логов через
`HostBridge`, stub-transport (`StdoutTransport`), placeholder-подпись. Нужно
зафиксировать целевое направление рефакторинга.

Сквозное решение о доверительной плоскости — в
[`cybercity/adr/0002-trust-boundary.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0002-trust-boundary.md);
сквозное решение о Rust + out-of-band — в
[`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md).
Этот ADR — локальное решение коллектора следовать им.

## Decision

Перевести коллектор из in-guest агента во внешний **out-of-band per-host**
наблюдатель:

- **Размещение:** коллектор работает на хосте (гипервизор / K8s-нода), наблюдает
  гостей **снаружи**, read-only, в mgmt-плоскости без маршрута из range.
  Структурно недосягаем для цели.
- **Зонды (read-only):** fs (ZFS-snapshot → ro-mount → integrity diff /
  overlay2-walk), net (port-mirror / eBPF-XDP → Zeek/Suricata), mem
  (`virsh dump-guest-memory` → Volatility / `/proc/<pid>/mem`), proc (`/proc`-walk,
  `lsns`), syscall (Falco / Tetragon — контейнеры).
- **Подпись:** события в подписанном конверте Ed25519 (key id + nonce +
  timestamp, replay-protection); `cybercity-engine` верифицирует.
- **Транспорт:** Kafka/Redpanda в mgmt-плоскости, mTLS + ACL на продюсеров;
  гости до брокера не достукиваются.
- **Control-канал:** от `cybercity-manage` (наблюдать цель / снапшот сейчас /
  обновить политику); действия над гостем (reset/изоляция) — через manage/фабрику,
  **не** через in-guest код.
- **In-guest обогащение:** опционально, best-effort, **никогда** не источник для
  scoring. «Агент замолчал» — сам по себе сигнал.
- **Нейминг:** `ccna-*` → `ccc-*` (cyber city collector); бинарник
  `cybercity-node-agent` → `cybercity-collector`; env-префикс `CCNA_` → `CCC_`.

## Consequences

### Positive

- Поток, которому можно доверять для scoring (вместе с hub ADR-0002).
- Зонды не зависят от гостевой ОС и не светятся в range.
- Read-only по конструкции: атакующий не может через коллектор воздействовать
  на цель.
- Чёткая граница ответственности: наблюдение — коллектор, действие — manage.

### Negative

- Зонды требуют прав на гипервизоре/узле; часть — привилегированные.
- Разнородные runtime цели (VM на Proxmox/ZFS vs контейнеры на K8s) требуют
  разных наборов зондов.
- Состояние кода сейчас — in-guest MVP; переход на out-of-band — отдельный
  заход: зонды, Ed25519-подпись, реальный Kafka-transport, канонический event
  envelope.

## Alternatives considered

- **In-guest агент (как сейчас):** отвергнут — ненадёжен в скомпрометированной
  среде; поток нельзя доверять для scoring.
- **Go/Python для коллектора:** возможны, но Rust даёт лучший профиль для
  per-host демона с доверием (строгая типизация, безопасная память, низкие
  накладные расходы на зонд). Сквозное решение — hub ADR-0003.
- **Только in-guest enrichment как источник scoring:** отвергнут — нарушает
  доверительную границу.

## Related

- [`cybercity/adr/0002-trust-boundary.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0002-trust-boundary.md) — доверительная граница (trusted vs best-effort).
- [`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md) — сквозное решение: Rust + out-of-band.
- [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md) — канон состава, «История переименований».
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md) — внутренняя архитектура (6-crate workspace).
- [`../DATA_FLOW.md`](../DATA_FLOW.md) — поток данных: зонд → конверт → Kafka → engine.