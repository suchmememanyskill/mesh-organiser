cmd /c build-web-svelte.bat
cd web

set LOCAL_ACCOUNT_PASSWORD=abc
set APP_CONFIG_PATH=Z:/config.json
set MESH_THUMBNAIL_EXECUTABLE_PATH=D:\Dev\mesh-organiser\external-binaries\mesh-thumbnail-x86_64-pc-windows-msvc.exe
::set RUST_BACKTRACE=full
cargo run