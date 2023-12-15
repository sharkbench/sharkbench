const http = require('http');
const express = require('express');

const app = express();
const port = 3000;

function fetchData(url, callback) {
    http.get(url, (res) => {
        let data = '';

        res.on('data', (chunk) => {
            data += chunk;
        });

        res.on('end', () => {
            callback(null, JSON.parse(data));
        });
    }).on('error', (err) => {
        callback(err, null);
    });
}

app.get('/api/v1/periodic-table/element', (req, res) => {
    const symbol = req.query.symbol;
    fetchData('http://web-data-source/data.json', (err, json) => {
        const entry = json[symbol];
        res.json({
            name: entry.name,
            number: entry.number,
            group: entry.group
        });
    });
});

app.get('/api/v1/periodic-table/shells', (req, res) => {
    const symbol = req.query.symbol;
    fetchData('http://web-data-source/data.json', (err, json) => {
        res.json({
            shells: json[symbol].shells
        });
    });
});

app.listen(port, () => {
    console.log(`Running on port ${port}`);
});
