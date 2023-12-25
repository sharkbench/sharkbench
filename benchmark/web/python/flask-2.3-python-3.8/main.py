from flask import Flask, request, Response
import json
import requests

app = Flask(__name__)
app.json.sort_keys = False

session = requests.Session()

def fetchData(url):
    with session.get(url) as response:
        response.raise_for_status()
        return response.json()

@app.route('/', methods=['GET'])
def index():
    return 'Hello world!'

@app.route('/api/v1/periodic-table/element', methods=['GET'])
def get_element():
    symbol = request.args.get('symbol')
    json_data = fetchData('http://web-data-source/data.json')
    entry = json_data[symbol]
    return Response(json.dumps({
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group']
    }, separators=(',', ':')), mimetype='application/json')

@app.route('/api/v1/periodic-table/shells', methods=['GET'])
def get_shells():
    symbol = request.args.get('symbol')
    json_data = fetchData('http://web-data-source/data.json')
    shells = json_data[symbol]['shells']
    return Response(json.dumps({'shells': shells}, separators=(',', ':')), mimetype='application/json')

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000)
