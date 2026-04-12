# inert-api

사진과 글을 공유하는 앱의 백엔드 API.

## 기술 스택

- [Rust](https://www.rust-lang.org/) (stable)
- [Poem](https://github.com/poem-web/poem) (HTTP 프레임워크)
- [SQLite](https://www.sqlite.org/) + [sqlx](https://github.com/launchbadge/sqlx)
- AWS S3 (Lightsail Object Storage)
- JWT (jsonwebtoken)
- Google OAuth

## 시작하기

```bash
cargo run
```

## 빌드

```bash
cargo build --release
```

## API 엔드포인트

### 공개

| 메서드 | 경로 | 설명 |
|--------|------|------|
| `POST` | `/auth/login/google` | Google ID token으로 로그인 (JWT 발급) |
| `GET` | `/feed` | 공개 피드 |
| `GET` | `/s/:slug` | still 상세 |
| `GET` | `/u/:username` | 유저 스틸 목록 |
| `GET` | `/u/:username/profile` | 유저 공개 프로필 |

### 인증 필요

| 메서드 | 경로 | 설명 |
|--------|------|------|
| `GET` | `/auth/me` | 내 정보 |
| `PUT` | `/auth/me` | 프로필 수정 |
| `GET` | `/stills/mine` | 내 스틸 목록 |
| `POST` | `/stills` | 스틸 생성 |
| `DELETE` | `/stills/:id` | 스틸 삭제 |
| `POST` | `/images/upload` | 이미지 업로드 |

## 배포

- GitHub Actions로 master 브랜치 push 시 자동 배포
- AWS Lightsail 인스턴스에서 직접 빌드
- nginx 리버스 프록시 + Let's Encrypt HTTPS

## 연결 프로젝트

- 프론트엔드: [inert](https://github.com/tyange/inert) (TanStack Start + React)
