# inert-api (백엔드)

Rust + Poem 프레임워크. cargo 사용.

## 기술 스택
- Rust (stable)
- Poem (HTTP 프레임워크)
- SQLite + sqlx
- AWS S3 (Lightsail Object Storage, Seoul)
- JWT (jsonwebtoken)
- Google OAuth (tokeninfo API 검증)

## 인프라
- AWS Lightsail: 54.180.157.98 (ap-northeast-2a, small_3_0)
- Object Storage: inert-storage (Seoul, public read)
- CDN: https://inert-storage.s3.ap-northeast-2.amazonaws.com
- 도메인: https://api.inert.tyange.com
- systemd 서비스: `inert-api`
- nginx 리버스 프록시 + Let's Encrypt HTTPS

## 배포
- GitHub Actions (`.github/workflows/deploy.yml`)
- master 브랜치 push 시 자동 배포
- 서버에서 직접 빌드 (cross-compilation 불가 — Mac arm64 → Linux x86_64)

## API 엔드포인트
- `POST /auth/login/google` — Google ID token → JWT
- `GET /auth/me` — 내 정보
- `GET /feed` — 공개 피드
- `GET /s/:slug` — still 상세
- `GET /u/:username` — 유저 스틸 목록
- `GET /stills/mine` — 내 스틸 목록 (인증 필요)
- `POST /stills` — 스틸 생성 (인증 필요)
- `DELETE /stills/:id` — 스틸 삭제 (인증 필요)
- `POST /images/upload` — 이미지 업로드 (인증 필요)

## DB 스키마 핵심
- `users`: user_id, email, username, google_sub, display_name, avatar_url
- `stills`: still_id, slug, user_id, caption, published_at
- `still_images`: image_id, still_id, image_url, image_key, position (다중 이미지 지원)

## 주요 결정사항
- still_images 별도 테이블로 분리 (다중 이미지, position 순서 관리)
- SQLite JSON 집계로 이미지 목록 조회 (JOIN 대신)
- Google 로그인만 지원 (이메일/비번 없음)

## 현재 상태
- 완료: 전체 API 구현 및 배포
- 완료: Google OAuth, JWT 인증
- 완료: 이미지 업로드 (S3)
- 완료: CI/CD (GitHub Actions)
