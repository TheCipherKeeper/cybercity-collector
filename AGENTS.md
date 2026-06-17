# AGENTS.md — правила для AI-агентов и контрибьюторов CyberCity Collector

## Иерархия документов (от старшего к младшему)

**Над репозиторием** — хаб `cybercity` держит системные документы:

- [`cybercity/COMPOSITION.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/COMPOSITION.md) — канон состава, контрактов, доверительной границы.
- [`cybercity/CONVENTIONS.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/CONVENTIONS.md) — кросс-репо конвенции (язык, скелет репо, ADR-формат, event envelope).
- [`cybercity/adr/`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/) — сквозные ADR (почему 6 репо, доверительная граница, Rust-коллектор).

**Внутри репозитория:**

1. `docs/adr/` — действующие архитектурные решения. ADR со статусом
   `superseded` не имеют силы.
2. `AGENTS.md` (этот файл) — операционные правила работы в репозитории.
3. `README.md` — краткое описание и quick start.
4. `docs/` — внутренняя документация (`ARCHITECTURE.md`, `DEVELOPMENT.md`,
   `DATA_FLOW.md`).
5. Код, тесты, конфиги — реализация принятых решений.

Если документы противоречат друг другу, побеждает старший. Любое расхождение —
повод создать новый ADR.

## Ключевые принципы

- **Out-of-band, не in-guest.** Наблюдатель живёт на привилегию выше цели
  (гипервизор / K8s-нода) и недосягаем из range-сегмента — tamper-proof по
  конструкции, а не по усилию. Текущий код — in-guest MVP; целевое состояние —
  out-of-band зонды (см. ADR-0001 и
  [`cybercity/adr/0003-collector-rust-out-of-band.md`](https://github.com/TheCipherKeeper/cybercity/blob/main/adr/0003-collector-rust-out-of-band.md)).
- **Read-only.** Коллектор не пишет в гости; действия (reset/изоляция) — через
  `cybercity-manage`/фабрику, не через in-guest код.
- **Подписанные события (Ed25519).** Каждый конверт подписан (key id + nonce +
  timestamp, replay-protection); `engine` верифицирует. Kafka — mTLS + ACL на
  продюсеров. Сейчас — placeholder; реализация в дорожной карте.
- **In-guest никогда не доверен.** Опциональное in-guest обогащение —
  best-effort, **никогда** не источник для scoring. «Агент замолчал» — сам по
  себе сигнал.
- **Target-агностичен: все runtime-виды единообразно.** Коллектор наблюдает
  `{vm, container, lite}` (umbrella ADR-0004) одним probe-pipeline; `honeypot` —
  purpose-флаг от manage, не отдельный режим. Класса «engine-synthesized service
  events» нет — движок регистратор, не симулятор (всё, что движок знает о цели,
  приходит подписанным наблюдением).
- **Crate-имена `ccc-*`.** Старые `ccna-*` / `cybercity-node-agent` сняты;
  бинарник — `cybercity-collector`; env-префикс — `CCC_`.

## Правила для AI-агентов

### Что агенту МОЖНО

- Писать Rust-код в `crates/` (новые зонды, transport, command-handlers).
- Редактировать `Cargo.toml` зависимости с обоснованием.
- Создавать новые ADR, если меняется архитектурное решение.
- Запускать `cargo build`, `cargo run --bin cybercity-collector`, `cargo test`,
  `cargo fmt --check`, `cargo clippy -- -D warnings`.
- Обновлять `README.md`, `AGENTS.md`, `docs/` при изменении структуры.

### Чего агенту НЕЛЬЗЯ

- Редактировать ADR без явного указания или создания нового ADR.
- Добавлять зависимости без обоснования в ADR или комментарии.
- Делать коммиты, пуши, PR — это делает человек.
- Фабриковать реализацию out-of-band зондов или Ed25519-подписи, которой нет в
  коде: описывать честно — что есть stub/placeholder, что — TODO.
- Трогать `Cargo.lock`, `config/`, `fixtures/`, `target/`, `.gitignore` без
  явного одобрения.

## Структура репозитория

```
cybercity-collector/
├── README.md                         # обзор + quick start
├── AGENTS.md                         # этот файл
├── CONTRIBUTING.md                   # тонкий указатель → docs/DEVELOPMENT.md
├── Cargo.toml                        # workspace (6 членов)
├── Cargo.lock                        # pinned deps
├── LICENSE                           # MIT
├── LICENSE-DOCS                      # CC BY 4.0
├── config/
│   └── example.toml                   # пример конфига → копируют в local.toml
├── crates/                           # workspace-члены
│   ├── ccc-core/                     # runtime, config, lifecycle, policy, envelope-типы
│   ├── ccc-host/                      # read-only доступ к хосту (target: out-of-band зонды)
│   ├── ccc-telemetry/                 # сбор событий из зондов
│   ├── ccc-kafka/                     # transport (stub/Stdout) + подпись конверта (placeholder)
│   ├── ccc-command/                   # control-команды от manage
│   └── ccc-node/                      # бинарник cybercity-collector (composition root)
├── fixtures/                         # тестовые данные логов
└── docs/                             # документация
    ├── ARCHITECTURE.md
    ├── DEVELOPMENT.md
    ├── DATA_FLOW.md
    └── adr/
```

## Рабочий цикл

1. Прочитать соответствующий ADR и текущий код.
2. Внести изменения.
3. Запустить `cargo build`, `cargo test`, (TODO) `cargo fmt --check`,
   `cargo clippy -- -D warnings`.
4. Показать результат пользователю. Не коммитить.

## Язык документации

Вся документация и ADR ведутся на русском языке. README может содержать
английские бейджи и ссылки, но основной текст — русский. Английский допустим
только для: бейджей, идентификаторов кода, имён crate'ов, значений поля
`Status:` ADR (`Accepted` / `Superseded` / `Amended`).