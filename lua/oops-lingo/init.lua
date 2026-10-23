local M = {}

local function root()
  local src = debug.getinfo(1, "S").source:sub(2)
  return vim.fn.fnamemodify(src, ":h:h:h")
end

local function package_version()
  local manifest = assert(io.open(root() .. "/Cargo.toml", "r"))
  local contents = manifest:read("*a")
  manifest:close()
  local version = contents:match('version%s*=%s*"([%w%.%+%-]+)"')
  return assert(version, "package version not found in Cargo.toml")
end


-- Синхронно выполняем внешнюю программу; массив аргументов избегает shell-quoting.
local function invoke(args, cwd)
  -- vim.system запускает процесс без shell; :wait() дожидается завершения.
  -- cwd может быть nil; text=true декодирует вывод как текст.
  local proc = vim.system(args, { cwd = cwd, text = true }):wait()
  -- Код 0 означает успех. При ошибке сохраняем stderr в сообщении.
  if proc.code ~= 0 then
    error(table.concat(args, " ") .. " failed:\n" .. (proc.stderr or proc.stdout or ""))
  end
  return proc.stdout or ""
end

-- Собираем cdylib локально через Cargo и возвращаем путь к результату.
local function build(os, release)
  local cmd = { "cargo", "build" }
  local dir = "debug"
  if release then
    table.insert(cmd, "--release")
    dir = "release"
  end
  -- Cargo сам решает, что требуется перекомпилировать; текущий каталог — корень проекта.
  invoke(cmd, root())
  -- У Rust cdylib имена платформозависимы; у macOS расширение .dylib.
  local filename = os == "windows" and "rust_hello_native.dll"
      or (os == "macos" and "librust_hello_native.dylib" or "librust_hello_native.so")

  return root() .. "/target/" .. dir .. "/" .. filename
end

local function platform()
  -- vim.uv — встроенный доступ к libuv; os_uname() возвращает сведения об ОС.
  local u = vim.uv.os_uname()
  -- Таблица используется как map: внешний sysname -> наше имя в GitHub Releases.
  local os = ({ Linux = "linux", Darwin = "macos", Windows_NT = "windows" })[u.sysname]
  -- Разные ОС называют одну архитектуру по-разному, нормализуем названия.
  local arch = ({ x86_64 = "x86_64", amd64 = "x86_64", aarch64 = "aarch64", arm64 = "aarch64" })[u.machine]
  -- Не пытаемся скачать случайный бинарник на неподдерживаемой платформе.
  assert(os and arch, "Unsupported platform: " .. u.sysname .. "/" .. u.machine)
  -- Для Lua-модуля под Linux/macOS используем .so, под Windows .dll.
  local ext = os == "windows" and "dll" or "so"
  -- Lua-функция может вернуть несколько значений.
  return os .. "-" .. arch, ext, os
end

function M.setup(opts)
  opts = opts or {}
  local release = not opts.debug
  local _, _, os = platform()
  local source = build(os, release)
  -- package.loadlib загружает нативную библиотеку и находит экспорт Lua-модуля.
  -- nvim-oxi экспортирует luaopen_rust_hello_native благодаря #[oxi::plugin].
  local loader, err = package.loadlib(source, "luaopen_oops_lingo")
  assert(loader, err)
  -- Запуск loader() инициализирует Rust-плагин и регистрирует :RustHello.
  loader()
end

return M
