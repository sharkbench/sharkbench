import requests
from django.http import JsonResponse

session = requests.Session()


def get_element(request):
    symbol = request.GET.get('symbol')
    json_data = session.get('http://web-data-source/element.json').json()
    entry = json_data[symbol]

    return JsonResponse({
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
    })


def get_shells(request):
    symbol = request.GET.get('symbol')
    json_data = session.get('http://web-data-source/shells.json').json()

    return JsonResponse({
        'shells': json_data[symbol],
    })
