const readline = require('readline');

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

const rl = readline.createInterface({
    input: process.stdin,
    terminal: false,
});

rl.on('line', (line) => {
    line = line.trim();
    if (line.length === 0) {
        return;
    }
    const iterations = parseInt(line);
    const result = calcPi(iterations);
    process.stdout.write(`${result[0]};${result[1]};${result[2]}\n`);
});
