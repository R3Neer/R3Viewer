# Complemento CodeAE integración SAMECLIENT01 R4

2026-10-10. SIN_CRITICAS_PENDIENTES respecto del ajuste exclusivamente de instrumentación. Complementa REVIEW-SAMECLIENT-R4.md SHA76DAA64DE1A24826B7AC7AA7646E265A17B03897C29F098BDDA4122374954253; no sustituye fuentes/binarios ni atribuye nuevo runtime.

Verificación física: src/syntax.rs canónico SHA8538D33389D38D3C2137AC861D2B680AB43FDABF6BD8F8E918F1ADD6AE1DDA9C sigue byteexacto al R4 aprobado. src/syntax_same_client_r2_tests.rs SHA59EE41EAAF0ED4FD7DDD0F813DDEE4495B5917500856F8C724FE4899D96BC13F difiere del módulo frozen A9F88D0A2C7AC7B2165EF52B6B195FA66F69498487AE19660B2517711FC34A20 exclusivamente en líneas48/57:
- store(true, Release) → SyntaxClient::own_close_pipe_fault(true).
- store(false, Release) → SyntaxClient::own_close_pipe_fault(false).

Cuerpo del helper cfg(test) syntax.rs658–660 realiza exactamente CLOSE_PIPE_FAULT.store(enabled, Ordering::Release), sin otra lógica, allocation, lock, operación nativa o efecto en autoridad. Orden respecto de pause/retry intacto. No cambio de fixtures, expectativas, plazos, quotas, selectors ignore ni protocolo de cleanup. La invocación hace utilizado el helper en build de tests y evita unusedhelper sin supresión.

Equivalencia comprobada por delta completo y cuerpo exacto, únicamente CodeAE. AE propia previa sobre fuente productiva R4 mantiene su alcance; nueva compilación/runtime/Clippyall de integración corresponde a los checks de raíz y no se declara ejecutada por este complemento. D4b/QA siguen fuera alcance. No edición canónica ni freeze realizada por revisor.