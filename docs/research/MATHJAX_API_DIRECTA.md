# MathJax: API directa y obligaciones del adaptador

2026-10-10. [Revisión independiente](../reviews/MathJax-api-directa-fuentes-r1.md) sin críticas pendientes para investigación exacta. Doce módulos MJS/60147bytes de @mathjax/src4.1.3 cotejados con tarball e inventario previos; sin importar o ejecutar código. La documentación oficial se identifica4.0 y no sustituye las fuentes4.1.3.

- El loader esm instala import(file) genérico si mathjax.asyncLoad es falsy. Antes de integrarlo hay que cerrar/preload las cargas reales de módulos y fuentes conservando cobertura; omitir paquetes TeX por sí solo no prueba cierre.
- convert crea MathItem con métricas suministradas y usa80*ex cuando containerWidth es null; convertPromise espera retries/promesas. No calcula las métricas del terminal ni aporta timeout/cancel/Job/EOF/reap.
- TeX puede devolver nodoerror mediante formatError. NoUndefined instala otro fallback que devuelve mtext literal sin lanzar TexError. El adaptador deberá medir y distinguir esos casos; SVG o promesa resuelta no bastan para clasificar éxito ni alimentar caché.
- w/h/d/pwidth, clamp, Yflip, currentColor y vertical-align requieren un puente medido de métricas/baseline. pwidth puede eliminar viewBox. Los métodos protected observados no pasan a API pública estable por existir en JS.
- Caches local/global y reset/clear limpian conjuntos distintos. Hay que inventariar estado entre fórmulas y referencias del SVG, sin asumir aislamiento o autosuficiencia por localcache.

El scanner lexical propio observó74imports y50dependencias directas distintas fuera de selección; no es cierre transitivo o dinámico. Raíz cotejó19/19 evidencias físicas propias sin discrepancia. Manifest independienteB6588CD0D036D5402D1F0C2FF0BA5CB150A43ADF6469CA011F579F469761D827; informe5BDCEBF5ADCD97366E1DD6E9A84F3AADA39B7C4F8044AAC5B3DE38E0A0D6E10D. Candidato privado mathjax-direct-api-sources-r1 manifest576FF4E6BF5946A71F20E924D2AF903E50DB83D5F823E7121A1656882EEDFBE8. Recibo raíz .tmp/root-math-mermaid-preparation-2026-10-10/MATHJAX-API-REVIEW-ROOT-COTEJO-R1.json.

Siguiente incremento: refinar contrato de carga/paquetes/fuentes, métricas/errores/cache, recursos y supervisor con identidad/cancel/deadline originales; después probar runtime frío, corpus, errores, cancelación/reap y render independiente. [Avisos NewCM](NEWCM_FUENTES_AVISOS.md) y [fuentes publicadas](MOTORES_FUENTES_PUBLICADAS.md) son evidencia previa relacionada. Diseño/engine/bundle/render/QA/Unix/RV303/501 siguen pendientes.
