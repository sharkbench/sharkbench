using Printf

function calc_pi(iterations)
    pi = 0.0
    denominator = 1.0
    sum = 0.0
    custom_number = 0.0

    for x in 0:(iterations - 1)
        if x % 2 == 0
            pi += 1 / denominator
        else
            pi -= 1 / denominator
        end
        denominator += 2

        # custom
        sum += pi
        mod_3 = x % 3
        if mod_3 == 0
            custom_number += pi
        elseif mod_3 == 1
            custom_number -= pi
        else
            custom_number /= 2
        end
    end

    pi *= 4
    return pi, sum, custom_number
end

function main()
    for line in eachline(stdin)
        line = strip(line)
        isempty(line) && continue
        iterations = parse(Int, line)

        pi, sum, custom_number = calc_pi(iterations)

        @printf(stdout, "%.16f;%.7f;%.16f\n", pi, sum, custom_number)
        flush(stdout)
    end
end

main()
