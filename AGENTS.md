# rplb 협업 규칙

## 이슈 규칙

- 새 이슈를 열기 전에 중복 이슈와 관련 PR을 검색한다.
- 제목은 해결할 문제 또는 달성할 결과를 간결하게 설명한다. 로드맵 작업이면
  본문에 해당 마일스톤(`M0`~`M9`)을 명시한다.
- 본문에는 다음을 포함한다.
  - 문제 또는 배경
  - 완료 조건
  - 포함 범위와 제외 범위
  - 검증 방법 및 선행 의존성
- 모든 이슈에는 정확히 하나의 우선순위 라벨을 붙인다.
  - `priority: high`: 현재 마일스톤을 막거나 심각한 안정성·보안 문제
  - `priority: medium`: 일반적인 계획 작업
  - `priority: low`: 연기 가능한 개선 작업
- 모든 이슈에는 최소 하나의 유형 라벨을 붙인다:
  `bug`, `feature`, `docs`, `chore`, `refactor`, `test`, `ci`,
  `dependencies`, `performance`, `security`.
- 여러 성격이 있으면 관련 유형 라벨을 함께 붙인다.
- 성능 또는 보안 영향이 있으면 `performance` 또는 `security`를 관련 유형
  라벨과 함께 추가한다.
- PR로 해결되는 이슈는 PR에서 연결한다. 병합 시 자동으로 닫아야 하면
  `Closes #<issue-number>`를 사용한다.

## Pull Request 규칙

- PR은 하나의 논리적 변경만 포함하고 `main`을 대상으로 한다. 관련 없는
  리팩터링이나 포맷 변경은 분리한다.
- 제목은 `<kind>: 간결한 설명` 형식을 사용한다. `kind`는 `feat`, `fix`,
  `refactor`, `test`, `ci`, `chore`, `docs` 중 하나로 한다.
- PR 본문에는 변경 목적, 핵심 변경 사항, 호환성 또는 설정 영향, 검증 명령과
  결과, 관련 이슈를 적는다. TCP 프록시 등 네트워크 동작을 변경했다면 수동
  스모크 검증 결과도 포함한다.
- 해당할 때만 유형 라벨(`bug`, `feature`, `docs`, `chore`, `refactor`,
  `test`, `ci`, `dependencies`, `performance`, `security`)을 붙인다. 새 기능
  PR에는 `feature`, 버그 수정 PR에는 `bug`, 문서 전용 PR에는 `docs`를
  사용한다. `priority: *` 라벨은 이슈 전용이며 PR에는 붙이지 않는다.
