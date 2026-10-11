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

function M.setup(opts)
  opts = opts or {}

  vim.notify("Setup oops-lingo plugin" .. root() .. " - " .. package_version(), vim.log.levels.INFO)
end

return M
