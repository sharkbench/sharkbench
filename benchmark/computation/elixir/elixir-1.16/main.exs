defmodule SimpleServer do
  def calculate_pi(iterations) do
    Enum.reduce(0..(iterations - 1), {0.0, 0.0, 0.0, 1.0}, fn x, {pi, sum, custom_number, denominator} ->
      new_pi = if rem(x, 2) == 0 do
        pi + 1.0 / denominator
      else
        pi - 1.0 / denominator
      end

      new_sum = sum + new_pi

      new_custom = case rem(x, 3) do
        0 -> custom_number + new_pi
        1 -> custom_number - new_pi
        2 -> custom_number / 2
      end

      {new_pi, new_sum, new_custom, denominator + 2}
    end)
    |> then(fn {pi, sum, custom, _} ->
      pi = pi * 4
      [pi, sum, custom]
    end)
  end

  def start do
    {:ok, socket} = :gen_tcp.listen(3000, [:binary, packet: :raw, active: false, reuseaddr: true])
    IO.puts "Running on port 3000"
    accept_loop(socket)
  end

  def accept_loop(socket) do
    {:ok, client} = :gen_tcp.accept(socket)
    spawn(fn -> handle_client(client) end)
    accept_loop(socket)
  end

  def handle_client(client) do
    case :gen_tcp.recv(client, 0) do
      {:ok, data} ->
        request = data |> to_string() |> String.split("\r\n") |> List.first()

        if request && String.match?(request, ~r/GET/) do
          [_, path_with_query, _] = String.split(request, " ")

          if String.contains?(path_with_query, "?") do
            [_, query] = String.split(path_with_query, "?")
            params = URI.decode_query(query)
            iterations = String.to_integer(params["iterations"] || "100")

            result = calculate_pi(iterations)
            response_body = Enum.join(result, ";")

            response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n#{response_body}"
            :gen_tcp.send(client, response)
          end
        end

      {:error, _} -> nil
    end

    :gen_tcp.close(client)
  end
end

SimpleServer.start()
