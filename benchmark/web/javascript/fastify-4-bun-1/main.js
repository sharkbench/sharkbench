const fastify = require('fastify')({
    logger: false,
});

const port = 3000;

fastify.get('/api/v1/periodic-table/element', async function (request, reply) {
    const symbol = request.query.symbol;
    const elementRes = await(await fetch('http://web-data-source/element.json')).json();
    const element = elementRes[symbol];

    reply.send({
        name: element.name,
        number: element.number,
        group: element.group
    });
});

fastify.get('/api/v1/periodic-table/shells', async function (request, reply) {
    const symbol = request.query.symbol;
    const shellsRes = await(await fetch('http://web-data-source/shells.json')).json();

    reply.send({
        shells: shellsRes[symbol]
    });
});

fastify.listen({
    host: '0.0.0.0',
    port,
});
