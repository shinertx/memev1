# This Dockerfile can be used for all Python-based data consumers
FROM python:3.11-slim
WORKDIR /app
COPY data_consumers/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY data_consumers/ .
# The CMD will be specified in the docker-compose.yml file