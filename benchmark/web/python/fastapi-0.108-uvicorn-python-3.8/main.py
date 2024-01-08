from fastapi import FastAPI, HTTPException, Query
import logging
import httpx

app = FastAPI()
async_client = httpx.AsyncClient()
logging.getLogger('uvicorn').disabled = True


@app.get('/api/v1/periodic-table/element')
async def get_element(symbol: str = Query()):
    response = await async_client.get('http://web-data-source/element.json')
    json_data = response.json()
    entry = json_data.get(symbol)

    return {
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
    }


@app.get('/api/v1/periodic-table/shells')
async def get_shells(symbol: str = Query()):
    response = await async_client.get('http://web-data-source/shells.json')
    json_data = response.json()

    return {
        'shells': json_data[symbol],
    }

if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host='0.0.0.0', port=3000)
