#!/bin/bash
set -e

SERVER="ubuntu@54.180.157.98"
KEY="$HOME/.ssh/inert-key.pem"
REMOTE_DIR="/opt/inert-api"

echo "=== inert-api 배포 시작 ==="

# 로컬에서 빌드
echo "[1/4] 빌드 중..."
cargo build --release

# 바이너리 전송
echo "[2/4] 바이너리 전송 중..."
scp -i "$KEY" target/release/inert-api "$SERVER:$REMOTE_DIR/inert-api"

# .env 전송 (처음 배포 시에만)
if [ "$1" == "--init" ]; then
  echo "[2-1] .env 전송 중..."
  scp -i "$KEY" .env "$SERVER:$REMOTE_DIR/.env"
fi

# 서비스 재시작
echo "[3/4] 서비스 재시작 중..."
ssh -i "$KEY" "$SERVER" "sudo systemctl restart inert-api"

# 상태 확인
echo "[4/4] 상태 확인..."
ssh -i "$KEY" "$SERVER" "sudo systemctl status inert-api --no-pager"

echo "=== 배포 완료 ==="
