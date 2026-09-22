#!/bin/bash
set -e

# Initialize certificates if needed
if [ ! -f "$RM_CERTS_DIR/server.crt" ]; then
    echo "Generating self-signed certificates..."
    python -c "
from remarkable.server import SyncServer
import asyncio
async def init():
    server = SyncServer(data_dir='$RM_DATA_DIR', certs_dir='$RM_CERTS_DIR')
    await server.ensure_certs()
asyncio.run(init())
"
fi

# Start server
echo "Starting remarkable-server on $RM_HOST:$RM_PORT..."
exec python -m remarkable.server \
    --host "$RM_HOST" \
    --port "$RM_PORT" \
    --data-dir "$RM_DATA_DIR" \
    --certs-dir "$RM_CERTS_DIR" \
    --log-level "$RM_LOG_LEVEL"
