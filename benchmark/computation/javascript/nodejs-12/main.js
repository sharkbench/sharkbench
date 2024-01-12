const http = require('http');
const url = require('url');

const PORT = 3000;

function calcPi(iterations) {
    let pi = 0.0;
    let denominator = 1.0;
    let sum = 0.0;
    let customNumber = 0.0;
    for (let x = 0; x < iterations; x++) {
        if (x % 2 === 0) {
            pi = pi + (1 / denominator);
        } else {
            pi = pi - (1 / denominator);
        }
        denominator = denominator + 2;

        // custom
        sum += pi;
        switch (x % 3) {
            case 0:
                customNumber += pi;
                break;
            case 1:
                customNumber -= pi;
                break;
            case 2:
                customNumber /= 2;
                break;
        }
    }
    pi = pi * 4;
    return [pi, sum, customNumber];
}

const server = http.createServer((req, res) => {
    const queryObject = url.parse(req.url, true).query;
    const iterations = parseInt(queryObject.iterations);

    res.writeHead(200, { 'Content-Type': 'text/plain' });
    const result = calcPi(iterations);
    res.end(`${result[0]};${result[1]};${result[2]}`);
});

server.listen(PORT, () => {
    console.log(`Server listening on port ${PORT}`);
});
