import express from "express";

const app = express();
const port = 3000;

app.get('/api/v1/periodic-table/element', async (req, res) => {
    const symbol = req.query.symbol;
    const elementRes = await(await fetch('http://web-data-source/element.json')).json();
    const element = elementRes[symbol];

    res.json({
        name: element.name,
        number: element.number,
        group: element.group
    });
});

app.get('/api/v1/periodic-table/shells', async (req, res) => {
    const symbol = req.query.symbol;
    const shellsRes = await(await fetch('http://web-data-source/shells.json')).json();

    res.json({
        shells: shellsRes[symbol]
    });
});

app.listen(port, () => {
    console.log(`Running on port ${port}`);
});
