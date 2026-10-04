defmodule BenchmarkWeb.Router do
  use BenchmarkWeb, :router

  pipeline :api do
    plug :accepts, ["json"]
  end

  scope "/api/v1/periodic-table", BenchmarkWeb do
    pipe_through :api

    get "/element", ElementController, :get_element
    get "/shells", ElementController, :get_shells
  end
end
