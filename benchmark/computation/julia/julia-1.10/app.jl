using HTTP
using Sockets
using Printf

const PORT = 3000

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

function handle_request(request::HTTP.Request)
    query = HTTP.URIs.queryparams(HTTP.URIs.URI(request.target))
    iterations = parse(Int, get(query, "iterations", "1"))

    pi, sum, custom_number = calc_pi(iterations)

    formatted_output = @sprintf("%.16f;%.7f;%.16f", pi, sum, custom_number)
    return HTTP.Response(200, formatted_output)
end

HTTP.serve(handle_request, "0.0.0.0", PORT; verbose=true)
