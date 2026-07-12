# rplb

Rust로 L4 로드 밸런서부터 L7 리버스 프록시까지 단계적으로 구현하는 프로젝트입니다.

## 프로젝트 목표

- Tokio의 비동기 네트워크 I/O를 실제 TCP 프록시로 익힙니다.
- 여러 로드 밸런싱 정책의 상태와 선택 규칙을 직접 구현하고 검증합니다.
- L4에서 만든 백엔드 선택 기반을 Hyper 1.x HTTP/1.1 프록시까지
  확장합니다.
- 각 단계는 자동 테스트와 실제 백엔드를 사용하는 스모크 테스트로
  완료 여부를 판단합니다.

## 로드맵 한눈에 보기

| 마일스톤 | 결과 |
| --- | --- |
| M0 | 비동기 네트워크 프로젝트 기반 |
| M1 | 단일 backend TCP 프록시 |
| M2 | Round Robin L4 로드 밸런서 |
| M3 | Smooth Weighted Round Robin |
| M4 | Least Connections |
| M5 | 복원력과 운영 기반 |
| M6 | TOML 설정과 CLI |
| M7 | Hyper 기반 HTTP/1.1 리버스 프록시 |
| M8 | Host와 path prefix 기반 L7 라우팅 |
| M9 | 최종 품질 검증 |

## 세부 로드맵

### M0. 프로젝트 기반

Tokio 기반으로 실행 가능한 프로젝트 골격과 TCP/HTTP 계층의 책임 경계를
만들고, 로그 초기화와 오류 반환 규칙을 정합니다.

### M1. 단일 backend TCP 프록시

client 연결마다 정적 backend로 붙여 `copy_bidirectional`로 양방향
스트리밍합니다. 연결 실패·half-close에도 accept loop는 계속 동작해야
합니다.

### M2. Round Robin

healthy backend를 순서대로 선택하는 Round Robin을 TCP 연결 처리와
분리해 구현합니다. 빈 pool은 panic 없이 명확한 실패로 처리합니다.

### M3. Smooth Weighted Round Robin

현재 가중치/전체 가중치 기반 smooth 선택으로 양의 정수 weight 비율에
수렴시키고, 연속 선택이 한 backend로 쏠리지 않게 합니다.

### M4. Least Connections

활성 연결이 가장 적은 healthy backend를 선택하고, RAII guard로 정상
종료·오류·취소 등 모든 경로에서 카운터를 정확히 복구합니다.

### M5. 복원력과 운영 기반

주기적 TCP health check로 backend 상태를 갱신하고, connect·idle
timeout과 graceful shutdown을 적용하며 연결·정책 결과를 구조화된
tracing으로 남깁니다.

### M6. TOML 설정과 CLI

TOML에서 listener·backend·weight·policy·timeout을 읽고, `rplb
check`/`rplb run` CLI로 검증을 통과한 설정만 서버를 시작합니다.

### M7. Hyper 기반 HTTP/1.1 리버스 프록시

HTTP/1.1 요청·응답을 전체 버퍼링 없이 스트리밍하고, hop-by-hop 헤더
제거와 forwarding 헤더 부여, 502/503/504 오류 응답을 구현합니다.

### M8. L7 라우팅

Host exact match와 path longest-prefix match로 route를 결정하고,
route별로 M2~M4에서 만든 분산 정책과 backend pool을 재사용합니다.

### M9. 최종 품질 검증

L4와 L7 전체 경로를 알고리즘 단위·통합·E2E 테스트와 품질 게이트(`cargo
fmt --check`, `cargo clippy -- -D warnings`, `cargo test`)로 검증하고,
실제 클라이언트로 수동 스모크 테스트를 수행합니다.

## 범위와 후속 확장

핵심 로드맵은 TCP 기반 L4와 HTTP/1.1 기반 L7에 집중합니다. 다음 기능은
현재 마일스톤에 포함하지 않고 후속 확장으로만 다룹니다.

- UDP
- TLS termination
- HTTP/2
- HTTP/3
- WebSocket
- CONNECT tunnel
- dynamic service discovery
- admin API
- Prometheus metrics

## 참고 자료

- [Tokio `TcpListener` API][tokio-listener]
- [Tokio `copy_bidirectional` API][tokio-copy]
- [Tokio graceful shutdown 주제][tokio-shutdown]
- [Hyper 1.x server 가이드][hyper-server]
- [Hyper 1.x client 가이드][hyper-client]
- [Hyper body 모듈][hyper-body]
- [RFC 9110: Connection 관련 intermediary 규칙][rfc-connection]
- [RFC 9110: Server Error 5xx][rfc-server-error]

[tokio-listener]: https://docs.rs/tokio/latest/tokio/net/struct.TcpListener.html
[tokio-copy]: https://docs.rs/tokio/latest/tokio/io/fn.copy_bidirectional.html
[tokio-shutdown]: https://tokio.rs/tokio/topics/shutdown
[hyper-server]: https://hyper.rs/guides/1/server/hello-world/
[hyper-client]: https://hyper.rs/guides/1/client/basic/
[hyper-body]: https://docs.rs/hyper/latest/hyper/body/
[rfc-connection]: https://www.rfc-editor.org/rfc/rfc9110#section-7.6.1
[rfc-server-error]: https://www.rfc-editor.org/rfc/rfc9110#section-15.6
