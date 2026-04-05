#!/bin/bash
set -e

echo "=== inert-api 서버 초기 셋업 ==="

# 패키지 업데이트
sudo apt-get update -y
sudo apt-get upgrade -y

# 필수 패키지 설치
sudo apt-get install -y curl git build-essential pkg-config libssl-dev nginx certbot python3-certbot-nginx

# Rust 설치
if ! command -v cargo &> /dev/null; then
  echo "Rust 설치 중..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
else
  echo "Rust 이미 설치됨: $(rustc --version)"
fi

source "$HOME/.cargo/env"

# 앱 디렉토리 생성
sudo mkdir -p /opt/inert-api/data
sudo chown -R ubuntu:ubuntu /opt/inert-api

echo "=== 셋업 완료 ==="
echo "다음 단계: deploy.sh 실행"
