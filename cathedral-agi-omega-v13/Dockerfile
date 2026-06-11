FROM python:3.10-slim

WORKDIR /app

RUN apt-get update && apt-get install -y gcc g++ python3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . /app

ENTRYPOINT ["python", "node_entrypoint.py"]