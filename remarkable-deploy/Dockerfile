# remarkable-server production deployment
# Multi-stage build for minimal image size

# Stage 1: Build dependencies
FROM python:3.12-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    libffi-dev \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Copy and install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Stage 2: Runtime image
FROM python:3.12-slim as runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    tini \
    && rm -rf /var/lib/apt/lists/*

# Copy virtualenv from builder
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Create non-root user
RUN useradd --create-home --shell /bin/bash remarkable \
    && mkdir -p /data /certs \
    && chown -R remarkable:remarkable /app /data /certs

# Copy application
COPY --chown=remarkable:remarkable remarkable/ ./remarkable/
COPY --chown=remarkable:remarkable entrypoint.sh .

# Default environment
ENV RM_DATA_DIR=/data \
    RM_CERTS_DIR=/certs \
    RM_HOST=0.0.0.0 \
    RM_PORT=8080 \
    RM_LOG_LEVEL=info

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:${RM_PORT}/health || exit 1

# Volumes for persistent data
VOLUME ["/data", "/certs"]

# Run as non-root
USER remarkable

EXPOSE 8080

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["./entrypoint.sh"]
