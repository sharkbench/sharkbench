import Koa from 'koa';
import Router from '@koa/router';
import http from 'http';

const app = new Koa();
const router = new Router({
    prefix: '/api/v1/periodic-table'
});
const port = 3000;

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

router.get('/element', async (ctx) => {
    const symbol = ctx.query.symbol;
    const elementRes = await fetch('http://web-data-source/element.json');
    const element = elementRes[symbol];

    ctx.body = {
        name: element.name,
        number: element.number,
        group: element.group
    };
});

router.get('/shells', async (ctx) => {
    const symbol = ctx.query.symbol;
    const shellsRes = await fetch('http://web-data-source/shells.json');

    ctx.body = {
        shells: shellsRes[symbol]
    };
});

app
    .use(router.routes())
    .use(router.allowedMethods());

app.listen(port, () => {
    console.log(`Running on port ${port}`);
});
