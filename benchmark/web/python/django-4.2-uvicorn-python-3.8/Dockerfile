FROM python:3.8

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 3000
CMD ["python", "-m", "uvicorn", "app.asgi:application", "--host", "0.0.0.0", "--port", "3000", "--workers", "1", "--log-level", "critical"]
