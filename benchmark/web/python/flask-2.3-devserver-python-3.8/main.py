from flask import Flask, request, Response
import logging
import requests

app = Flask(__name__)
app.json.sort_keys = False

log = logging.getLogger('werkzeug')
log.disabled = True

session = requests.Session()


@app.route('/api/v1/periodic-table/element', methods=['GET'])
def get_element():
    symbol = request.args.get('symbol')
    json_data = session.get('http://127.0.0.1:5002/element.json').json()
    entry = json_data[symbol]

    return {
        'name': entry['name'],
        'number': entry['number'],
        'group': entry['group'],
    }


@app.route('/api/v1/periodic-table/shells', methods=['GET'])
def get_shells():
    symbol = request.args.get('symbol')
    json_data = session.get('http://127.0.0.1:5002/shells.json').json()

    return {
        'shells': json_data[symbol],
    }


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001, debug=False)
