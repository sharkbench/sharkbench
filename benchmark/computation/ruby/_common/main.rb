require 'socket'

def calculate_pi(iterations)
  pi = 0.0
  denominator = 1.0
  sum = 0.0
  custom_number = 0.0

  iterations.times do |x|
    if x.even?
      pi += 1.0 / denominator
    else
      pi -= 1.0 / denominator
    end

    denominator += 2

    # custom
    sum += pi
    case x % 3
    when 0
      custom_number += pi
    when 1
      custom_number -= pi
    when 2
      custom_number /= 2
    end
  end

  pi *= 4
  [pi, sum, custom_number]
end

server = TCPServer.new('0.0.0.0', 3000)
puts 'Running on port 3000'

loop do
  client = server.accept
  request_line = client.gets
  next unless request_line

  path, query = request_line.split[1].split('?')
  params = query.split('&').map { |param| param.split('=') }.to_h
  iterations = params['iterations'].to_i

  result = calculate_pi(iterations)
  client.puts "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n#{result.join(';')}"
  client.close
end
