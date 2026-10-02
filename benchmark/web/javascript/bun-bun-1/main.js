Bun.serve({
    port: 5001,
    fetch: async (req) => {
        const url = new URL(req.url);

        // Also check method to make it fair
        if (req.method !== "GET") return new Response("405!");

        const symbol = url.searchParams.get('symbol');
        switch (url.pathname) {
            case '/api/v1/periodic-table/element':
                const elementRes = await(await fetch('http://127.0.0.1:5002/element.json')).json();
                const element = elementRes[symbol];
                return new Response(JSON.stringify({
                    name: element.name,
                    number: element.number,
                    group: element.group
                }));
            case '/api/v1/periodic-table/shells':
                const shellsRes = await(await fetch('http://127.0.0.1:5002/shells.json')).json();
                return new Response(JSON.stringify({
                    shells: shellsRes[symbol]
                }));
        }

        return new Response("404!");
    },
});
