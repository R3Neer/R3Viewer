# Adendum focal R2: límite individual del assembly MathJax

**SIN_CRITICAS_PENDIENTES, sólo delta de diseño R2** SHA16F58D11360B1BE7E4C3200EA5BE88DF9AA79FD3A8BA4E02C4AD96C4E127EC8C. Se mantienen MC01–08 y dictamen R1 intactos. Sin assembly, escritura de módulos, Node/import/engine o runtime.

Comparación propia exacta del cuerpo previo al apéndice: sólo título R1→R2 y frase de límite256KiB con excepción→2MiB. El apéndice declara el motivo prospectivo; el resto del cuerpo se conserva. Capacidades64MiB por paquete/128MiB total, guardian, relojes30s+cleanup5s, corpus y alcance experimental no cambian.

Recuento propio de inventarios publicados ya contrastados, seleccionando package/package.json + prefijo package/mjs/: src1214 archivos/4291018B/max192556; NewCM464/16555159B/max1097462. NewCM tiene13 archivos SVGfontdata sobre262144B; ninguno de ambos conjuntos supera2097152B. Sumados20846177B, bajo128MiB; cada paquete bajo64MiB. Se coteja lectura de inventarios, no ejecución del conjunto, closure o uso real de todos los módulos.

El cambio evita omitir los rangos grandes de NewCM para pasar una cota anterior; no modifica límites de producto. Conserva fallo futuro ante una dependencia/capacidad no contemplada y exige revisión prospectiva antes de ampliar. La identidad y capacidad de cada archivo real deben revalidarse durante assembly; las garantías de ruta/exports/closure siguen sujetas al código y controles de A. Guardian B y los oráculos CP pendientes siguen gates antes de import C, sin aceptación de engine/distribución/QA/RV por asociación.

Resultado propio: RESULT-FOCAL-R2.json. No se alteraron candidatos ni evidencia R1.