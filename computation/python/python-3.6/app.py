from flask import Flask, request

app = Flask(__name__)

@app.route('/')
def index():
    iterations = int(request.args.get('iterations'))
    return str(calc_pi(iterations))

def calc_pi(iterations):
    pi = 0.0
    denominator = 1.0
    for x in range(iterations):
        if x % 2 == 0:
            pi = pi + (1 / denominator)
        else:
            pi = pi - (1 / denominator)
        denominator = denominator + 2
    pi = pi * 4
    return pi

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
