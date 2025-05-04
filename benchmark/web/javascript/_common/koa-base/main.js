import Koa from 'koa';
import Router from '@koa/router';

const app = new Koa();
const router = new Router({
    prefix: '/api/v1/periodic-table'
});
const port = 3000;

router.get('/element', async (ctx) => {
    const symbol = ctx.query.symbol;
    const elementRes = await(await fetch('http://web-data-source/element.json')).json();
    const element = elementRes[symbol];

    ctx.body = {
        name: element.name,
        number: element.number,
        group: element.group
    };
});

router.get('/shells', async (ctx) => {
    const symbol = ctx.query.symbol;
    const shellsRes = await(await fetch('http://web-data-source/shells.json')).json();

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
