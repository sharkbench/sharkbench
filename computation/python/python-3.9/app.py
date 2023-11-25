from flask import Flask, request
import bcrypt

app = Flask(__name__)

@app.route('/')
def index():
    password = request.args.get('password')
    salt = request.args.get('salt')

    hashed_password = bcrypt.hashpw(password.encode('utf-8'), salt.encode('utf-8')).decode('utf-8')
    return hashed_password

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000, debug=True)
