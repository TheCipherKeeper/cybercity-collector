# ccc-command

Приём и исполнение команд от cybercity-manage. Проверка политики,
проверка подписи, tamper detection.

## Интерфейсы

- `CommandExecutor::new(policy, bridge, lifecycle)` — создание.
- `CommandExecutor::execute(cmd: CommandEnvelope) -> Result<AgentEvent, CommandError>`
  — исполнение команды.

## Типы

```rust
pub enum CommandError {
    NotAllowed(String),
    MissingField(String),
    Host(ccc_host::HostError),
    InvalidSignature,
}

pub struct CommandExecutor {
    policy: Arc<Policy>,
    bridge: Arc<HostBridge>,
    lifecycle: Arc<Lifecycle>,
}
```

## Что есть

- execute: проверка policy.can_run_command → match kind → result.
- При неразрешённом kind → record_tamper() + NotAllowed.
- handle_status: возвращает AgentEvent("command_status") с state и
  tamper_count.
- handle_read_file: читает путь из payload["path"] через HostBridge,
  возвращает AgentEvent("command_read_file") с size и preview (256 символов).
- Unknown kind → NotAllowed.

## Что TODO

- Верификация подписи команды (Ed25519). Сейчас InvalidSignature возможен,
  но не проверяется крипто.
- Расширение набора команд: snapshot, update_policy, observe (команда на
  обновление observation manifest).
- Приём observation manifest через command-канал.
- Логирование аудита (cc.audit topic) для каждой команды.
- Rate limiting: ограничение частоты команд от manage.

## Ограничения

- Разрешённые kinds: только из policy.allowed_command_kinds.
- Сейчас поддерживаются: "status", "read_file".
- Проверка подписи — placeholder (signature presence, не крипто).
- read_file возвращает preview 256 символов — для больших файлов нужен
  pagination/offset.

## Зависимости

ccc-core, ccc-host, ccc-kafka, tokio, tracing, thiserror, serde_json