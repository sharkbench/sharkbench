import httpx
from django.http import JsonResponse

async_client = httpx.AsyncClient()


async def get_element(request):
    symbol = request.GET.get('symbol')
    response = await async_client.get('http://web-data-source/element.json')
    json_data = response.json()
    entry = json_data[symbol]

    return JsonResponse({
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
    })


async def get_shells(request):
    symbol = request.GET.get('symbol')
    response = await async_client.get('http://web-data-source/shells.json')
    json_data = response.json()

    return JsonResponse({
        'shells': json_data[symbol],
    })
