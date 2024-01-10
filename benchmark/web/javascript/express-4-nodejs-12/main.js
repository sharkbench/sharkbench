const http = require('http');
const express = require('express');

const app = express();
const port = 3000;

// fetch() is not available prior Node.js 18, and it is slower than http.get()
// TODO: Use fetch() as soon as the following issue is resolved:
// https://github.com/nodejs/undici/issues/1203
function fetch(url) {
    return new Promise((resolve, reject) => {
        http.get(url, (res) => {
            let data = '';

            res.on('data', (chunk) => {
                data += chunk;
            });

            res.on('end', () => {
                resolve(JSON.parse(data));
            });
        }).on('error', (err) => {
            reject(err);
        });
    });
}

app.get('/api/v1/periodic-table/element', async (req, res) => {
    const symbol = req.query.symbol;
    const elementRes = await fetch('http://web-data-source/element.json');
    const element = elementRes[symbol];

    res.json({
        name: element.name,
        number: element.number,
        group: element.group
    });
});

app.get('/api/v1/periodic-table/shells', async (req, res) => {
    const symbol = req.query.symbol;
    const shellsRes = await fetch('http://web-data-source/shells.json');

    res.json({
        shells: shellsRes[symbol]
    });
});

app.listen(port, () => {
    console.log(`Running on port ${port}`);
});
