from flask import Flask, request, Response
import logging
import requests

app = Flask(__name__)
app.json.sort_keys = False

log = logging.getLogger('werkzeug')
log.disabled = True

session = requests.Session()

def fetchData(url):
    response = session.get(url)
    return response.json()

@app.route('/api/v1/periodic-table/element', methods=['GET'])
def get_element():
    symbol = request.args.get('symbol')
    json_data = fetchData('http://web-data-source/data.json')
    entry = json_data[symbol]

    return {
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
    }

@app.route('/api/v1/periodic-table/shells', methods=['GET'])
def get_shells():
    symbol = request.args.get('symbol')
    json_data = fetchData('http://web-data-source/data.json')

    return {
        'shells': json_data[symbol]['shells'],
    }

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=False)
