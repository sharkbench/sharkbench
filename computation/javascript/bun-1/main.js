const url = require('url');

function calcPi(iterations) {
    let pi = 0.0;
    let denominator = 1.0;
    for (let x = 0; x < iterations; x++) {
        if (x % 2 === 0) {
            pi = pi + (1 / denominator);
        } else {
            pi = pi - (1 / denominator);
        }
        denominator = denominator + 2;
    }
    pi = pi * 4;
    return pi;
}

const server = Bun.serve({
    port: 3000,
    fetch(request) {
        const queryObject = url.parse(request.url, true).query;
        const iterations = parseInt(queryObject.iterations);
        return new Response(`${calcPi(iterations)}`);
    },
});

console.log(`Listening on localhost: ${server.port}`);
