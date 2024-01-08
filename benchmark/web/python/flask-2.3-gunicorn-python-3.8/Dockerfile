FROM python:3.8-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 3000
CMD ["gunicorn", "--bind", "0.0.0.0:3000", "--workers=4", "--reuse-port", "main:app"]
