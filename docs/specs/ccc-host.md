# ccc-host

Read-only доступ к хосту. Единственная точка, через которую коллектор
обращается к файловой системе и ресурсам хоста. Защита от path traversal,
проверка политики.

## Интерфейсы

- `HostBridge::new(policy: Policy)` — создание с политикой.
- `HostBridge::read_file(path) -> Result<Vec<u8>, HostError>` — чтение файла,
  если разрешён политикой.
- `HostBridge::list_dir(path) -> Result<Vec<PathBuf>, HostError>` — список
  записей в директории, если разрешён политикой.
- `HostBridge::policy() -> &Policy` — доступ к политике.

## Типы

```rust
pub enum HostError {
    NotAllowed(PathBuf),
    Traversal(PathBuf),
    Io(std::io::Error),
}

pub struct HostBridge {
    policy: Policy,
}
```

## Что есть

- read_file: нормализация пути → проверка политики → чтение.
- list_dir: нормализация → проверка политики → чтение директории.
- normalize: отклонение `..` компонентов, каноникализация symlink'ов,
  fallback на clean() если canonicalize не работает.
- PathClean trait для безопасного разрешения `..` без обращения к FS.
- Тест: rejects_traversal (проверяет `/var/log/../etc/passwd`).

## Что TODO

- Out-of-band зонды (вместо file-tail MVP):
  - fs: ZFS-snapshot → ro-mount → integrity diff
  - net: port-mirror / eBPF-XDP → Zeek/Suricata
  - mem: virsh dump-guest-memory → Volatility
  - proc: /proc-walk, lsns
  - syscall: Falco / Tetragon (контейнеры)
- Probe trait: унифицированный интерфейс для всех зондов.
- Поддержка runtime_kind: разные зонды для vm/container/lite.
- Интеграция с observation manifest: активация зондов по конфигу manifest.

## Ограничения

- Только read-only: никаких write/exec напрямую через HostBridge.
- Path traversal: `..` компоненты отклоняются до обращения к FS.
- Каноникализация: symlink'ы резолвятся через tokio::fs::canonicalize,
  fallback на ручную clean() если файл не существует.
- Policy.can_read_file использует starts_with на канонизированном пути.

## Зависимости

ccc-core, tokio, tracing, thiserror