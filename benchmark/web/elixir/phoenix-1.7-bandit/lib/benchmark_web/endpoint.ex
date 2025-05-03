defmodule BenchmarkWeb.Endpoint do
  use Phoenix.Endpoint, otp_app: :benchmark

  plug Plug.Parsers,
    parsers: [],
    pass: ["*/*"]

  plug BenchmarkWeb.Router
end
