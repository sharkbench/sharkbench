const http = require('http');
const fastify = require('fastify')({
    logger: false,
});

const port = 5001;

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

fastify.get('/api/v1/periodic-table/element', async function (request, reply) {
    const symbol = request.query.symbol;
    const elementRes = await fetch('http://127.0.0.1:5002/element.json');
    const element = elementRes[symbol];

    reply.send({
        name: element.name,
        number: element.number,
        group: element.group
    });
});

fastify.get('/api/v1/periodic-table/shells', async function (request, reply) {
    const symbol = request.query.symbol;
    const shellsRes = await fetch('http://127.0.0.1:5002/shells.json');

    reply.send({
        shells: shellsRes[symbol]
    });
});

fastify.listen({
    host: '0.0.0.0',
    port,
});
