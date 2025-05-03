defmodule BenchmarkWeb.ElementController do
  use BenchmarkWeb, :controller

  def get_element(conn, %{"symbol" => symbol}) do
    case HTTPoison.get("http://web-data-source/element.json") do
      {:ok, %HTTPoison.Response{status_code: 200, body: body}} ->
        json_data = Jason.decode!(body)
        entry = Map.get(json_data, symbol)

        if entry do
          conn
          |> json(%{
            "name" => entry["name"],
            "number" => entry["number"],
            "group" => entry["group"]
          })
        else
          conn
          |> put_status(404)
          |> json(%{error: "Element not found"})
        end

      {:error, %HTTPoison.Error{}} ->
        conn
        |> put_status(500)
        |> json(%{error: "Failed to fetch element data"})
    end
  end

  def get_shells(conn, %{"symbol" => symbol}) do
    case HTTPoison.get("http://web-data-source/shells.json") do
      {:ok, %HTTPoison.Response{status_code: 200, body: body}} ->
        json_data = Jason.decode!(body)
        shells = Map.get(json_data, symbol)

        if shells do
          conn
          |> json(%{
            "shells" => shells
          })
        else
          conn
          |> put_status(404)
          |> json(%{error: "Shells data not found for this element"})
        end

      {:error, %HTTPoison.Error{}} ->
        conn
        |> put_status(500)
        |> json(%{error: "Failed to fetch shells data"})
    end
  end

  def get_element(conn, _params) do
    conn
    |> put_status(400)
    |> json(%{error: "Missing required parameter: symbol"})
  end

  def get_shells(conn, _params) do
    conn
    |> put_status(400)
    |> json(%{error: "Missing required parameter: symbol"})
  end
end
