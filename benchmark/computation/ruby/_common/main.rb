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

$stdout.sync = true

while (line = $stdin.gets)
  line = line.strip
  next if line.empty?

  iterations = line.to_i
  result = calculate_pi(iterations)
  $stdout.puts result.join(';')
end
