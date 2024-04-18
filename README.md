
# Front (comsys-wasm)
- install rust and cargo
- install nginx
- В папке ./comsys-wasm
- `cargo install trunk --locked`
- `trunk build --release`
- Скомпилированные html, css, wasm и js файлы будут в папке ./dist в корневике репозитория
- Запустите nginx с конфигом из папки /nginx: `nginx -c .\ngnix.conf`
- Стандартно доступ к фротну идёт через https://127.0.0.1

# Back (comsys-server-rs)
- install rust and cargo
- `cargo run --release` для компиляции и запуска. хостится на локальном уровне http://127.0.0.1:50051
- При запуске nginx с конфигом из папки /nginx: `nginx -c .\ngnix.conf` имеем доступ к беку через https://127.0.0.1/api/, без порта

Примечания
- Без https бэк будет игнорировать запросы по правилам CORS
- Установка AccessToken для GrpcWeb client осуществляется в HttpOnly cookies
- Установка AuthToken всегда включается либо в запрос, либо в metadata. См. протоколы
- **protobuf**-файлы лежат в `./comsys-wasm/proto`
- Важно, чтобы при компиляции proto файлов в код сервера не искажались URL методов ! Нельзя допускать дополнительных вложений пакетов !
