const express = require('express');
const bcrypt = require('bcryptjs');

const app = express();
const port = 3000;

app.get('/', (req, res) => {
    const iterations = req.query.iterations;
    res.send(pi(iterations).toString());
});

function pi(iterations) {
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

app.listen(port, () => {
    console.log(`Running on http://localhost:${port}`);
});
