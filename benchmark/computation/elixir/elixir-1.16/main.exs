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
    case IO.gets("") do
      :eof ->
        :ok

      {:error, _} ->
        :ok

      line ->
        case String.trim(line) do
          "" ->
            nil

          input ->
            iterations = String.to_integer(input)
            result = calculate_pi(iterations)
            IO.puts(Enum.join(result, ";"))
        end

        start()
    end
  end
end

SimpleServer.start()
