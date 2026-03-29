# Contributing

## Окружение

Проект использует NixOS devshell. Все зависимости описаны в `flake.nix`:

```bash
nix develop
```

Основные инструменты: Rust (edition 2024), bun, cargo-tauri, DuckDB.

## Структура

```
crates/
  flibrarian-core/   # Общая библиотека: индексация, поиск, извлечение
  flibrarian-cli/    # CLI бинарник
  flibrarian-web/    # Axum веб-сервер (встраивает фронтенд через rust-embed)
  flibrarian-gui/    # Tauri десктоп-приложение
frontend/            # Svelte 5 + Vite (общий фронтенд для web и gui)
```

## Сборка

```bash
cargo build                    # dev-сборка (без bundled-duckdb!)
just gui-dev                   # GUI с hot-reload
just web-dev                   # Веб-сервер с фронтендом из disk
```

**Не используйте `--features bundled-duckdb` при разработке** — это расходует RAM и нужно только для продакшн-сборок.

## Проверки

```bash
just check                     # Все проверки: fmt, clippy, тесты, линтеры
just check-rust                # Только Rust
just check-frontend            # Только фронтенд
```

Перед коммитом автоматически запускаются pre-commit хуки (clippy + rustfmt).

## Стиль кода

- Clippy pedantic + nursery, `unsafe` запрещён
- Без комментариев в коде (кроме объяснения неочевидного "зачем")
- Функции 5–25 строк, максимум 50
- `anyhow` для ошибок приложения, `thiserror` для библиотечных
- Rust edition 2024

## Тесты

```bash
cargo test                     # Все тесты
cargo test test_name           # Один тест
```

Тесты core-крейта лежат в `crates/flibrarian-core/tests/` и работают напрямую с парсером (без БД).

## Релиз

Релизы создаются автоматически при пуше тега `v*`:

```bash
git tag v0.2.0
git push github v0.2.0
```

CI соберёт бинарники CLI, веб-сервера и GUI для всех платформ и создаст GitHub Release.
