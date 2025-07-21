FROM python:3.11-slim
WORKDIR /app

COPY strategy_factory/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY strategy_factory/factory.py .
CMD ["python", "factory.py"]
