# Integración de D2a R2: recorrido, perfiles y navegación

2026-10-06. Base compartida adee0072ceb8d06c74ae21b3646e28d8e3586393. Se integra el candidato D2a R2, manifiesto 73C19AE35C494248FFC94AFCE7139C1EA95CCDB8AE8A70BFFCAB2CF7ABAFBC84, 415 registros. Las dieciséis rutas del parche coinciden byte a byte con la entrega antes de preparar el índice. Las APIs Source del commit base se conservan.

## Comportamiento añadido

La biblioteca recorre los Items validados en su orden real, mantiene posiciones ligadas a identidad y prepara extremos sin repetir el recorrido desde la raíz durante su consulta. Los callouts colapsados ocultan el cuerpo y una expansión prepara otra identidad Flow. Home/End y movimientos ordinarios conservan el historial; los saltos con retorno admiten hasta 128 entradas reales y rechazan la siguiente sin cambiar el estado vigente.

Cada cursor retenido tiene propietario y cargo de memoria/checkpoint hasta su destrucción, incluso si antes se destruye el índice. Perfiles y candidatos usan el mismo ConsumerBudget; las consultas cobran trabajo antes de copiar o inspeccionar. El límite controla las allocations explícitas del consumidor, no toda la memoria del proceso.

## Revisión y comprobaciones

[El adversario independiente](Markdown-D2a-acceptance-r2.md) concluye SIN_CRITICAS_PENDIENTES para Flow, perfiles y navegación lógica: siete grupos propios aprobados sobre su ejecutable exacto, 0.03 s. Su manifiesto de evidencia es 675B5D362FBEB884F0A579F155341020A103DD1A79168C458D18A65060F424A6. Conserva los intentos fallidos del harness y la corrección de su grafo, sin alterar las expectativas ni el producto. La corrección de ownership de R1 y sus evidencias anteriores quedan preservadas fuera de Git.

En la combinación compartida pasan quince pruebas D2a pequeñas (0.07 s), cuatro regresiones D1 (0.00 s) y siete Source (0.77 s). Formato y Clippy de workspace/all-targets terminan con código 0. Logs temporales Markdown-D2a-shared-small-tests-r2.log, Markdown-D2a-shared-D1-regression-r2.log, Markdown-D2a-shared-Source-regression-r2.log y Markdown-D2a-shared-clippy-r2.log. El caso gigante ya acreditado por el autor no se repite por esta combinación: las modificaciones posteriores solo afectan APIs Source y documentación, y no cambia el algoritmo de extremos. Ese antecedente no se atribuye al adversario.

Las reglas de atributos preservan los bytes de fixtures y manifiestos D2a en cualquier checkout. src/lib.rs se normaliza de CRLF a LF según la política existente; contiene declaraciones de módulos, sin cambio semántico. El resto de las dieciséis rutas mantiene la identidad revisada en el índice.

## Alcance que sigue pendiente

Esta integración es una biblioteca: no conecta presentación Markdown, renderer, búsqueda/acciones Source, teclas o sesión TUI. D2b, D2c, routing/inline D3 y bloques Syntect D4 continúan abiertos. No acredita QA visual, gráficos, Unix, distribución ni cierre de RV-301 o del producto.
