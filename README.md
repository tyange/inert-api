# inert-api

사진과 글을 공유하는 앱의 백엔드 API.

## 기술 스택

- [Rust](https://www.rust-lang.org/) (stable)
- [Poem](https://github.com/poem-web/poem) (HTTP 프레임워크)
- [SQLite](https://www.sqlite.org/) + [sqlx](https://github.com/launchbadge/sqlx) (컴파일 타임 쿼리 검증)
- AWS Lightsail Object Storage (S3 호환) + CDN
- JWT (jsonwebtoken)
- Google OAuth (ID token 서버 측 검증)
- `image` crate (JPEG/PNG/WebP 디코딩 및 검증)

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
| `POST` | `/auth/login/google` | Google `id_token` → 서버 검증(tokeninfo) → JWT 발급 + 기존 계정 병합 |
| `GET` | `/feed` | 공개 피드 |
| `GET` | `/s/:slug` | still 상세 (다중 이미지 포함) |
| `GET` | `/u/:username` | 유저 스틸 목록 |
| `GET` | `/u/:username/profile` | 유저 공개 프로필(`bio` 최대 100자) |

### 인증 필요

| 메서드 | 경로 | 설명 |
|--------|------|------|
| `GET` | `/auth/me` | 내 정보 |
| `PUT` | `/auth/me` | 프로필 수정 (display_name, bio, avatar_url) |
| `GET` | `/stills/mine` | 내 스틸 목록 |
| `POST` | `/stills` | 스틸 생성 (multi-image body) |
| `DELETE` | `/stills/:id` | 스틸 삭제 |
| `POST` | `/images/upload` | 이미지 업로드 (multipart, 서버 검증 후 S3 + CDN) |

## 데이터 모델

```
users        : user_id, email, username, google_sub, display_name, avatar_url, bio(≤100)
stills       : still_id, slug, user_id, caption, published_at
still_images : image_id, still_id, image_url, image_key, position
```

`still_images`는 **`position` 컬럼으로 순서를 보존**하는 별도 테이블로 분리해 다중 이미지를 1:N으로 관리한다. 스틸 단건 조회 시 JOIN 후 position 오름차순으로 배열 직렬화.

## 주요 설계

- **에러 응답 JSON 표준화**: 모든 오류를 `{ "error": "..." }` 형태로 통일. 과거에는 일부 경로가 plain text로 내려가 프론트의 JSON 파서가 터지는 문제가 있었다.
- **S3 ACL 정책 전환**: 개별 오브젝트에 `public-read` ACL을 붙이지 않고, 버킷 정책 + **BlockPublicAcls** 를 유지한 채 **CDN URL**만 외부 공개한다. 초기에 개별 object ACL로 공개하던 구조를 AWS의 기본 Block Public ACLs 정책과 충돌하지 않게 정리한 결과다.
- **Google 로그인 흐름**: 프론트엔드가 받은 `id_token`을 서버가 Google `tokeninfo` 엔드포인트로 재검증 → `google_sub`으로 기존 유저를 찾고 없으면 프로필을 생성, JWT 발급.
- **이미지 업로드 검증**: `image` crate로 실제 디코드를 성공해야 저장(content-type 위조 방지). 실패 시 S3 업로드 자체를 스킵.

## 배포

- GitHub Actions `push` on `master` → Lightsail 인스턴스에서 **직접 `cargo build --release`** (Mac arm64 ↔ Linux x86_64 크로스컴파일을 피하기 위함) → systemd `inert-api` 서비스 재시작 → nginx 리버스 프록시 + Let's Encrypt HTTPS

## 테스트

```bash
cargo test
```

## 연결 프로젝트

- 프론트엔드: [inert](https://github.com/tyange/inert) (TanStack Start + React)
