# Dictamen independiente PDF wire/geometría R5

SIN_CRITICAS_PENDIENTES, exclusivamente D0 del candidato PROPOSAL-P1-WIRE-R5.md SHA256 6C0E0A40A7692876DB6BBE23BB0DD04562B46E8098AEB36A8D783432E0E9FB91 (21415 bytes).

Criterios prospectivos propios CRITERIA.md SHA256 382F1377695433F90DF55341F52B2CBA9F2B449BBA3FFB5E694FAEC75C3CE545 fijados y comunicados antes de leer candidato u outputs. Manifest R5 SHA256 6152211138CF53073F4F3F86CCD11DAA9C4FA7F58A7045F2472F54EFD1D1C082: sus15 entradas contrastadas físicamente SHA/bytes, todas coinciden. El hash del candidato fue comprobado en su primera llamada de lectura; la comprobación completa de las15 entradas se efectuó en la llamada siguiente. No atribuir al registro un orden más fuerte que éste. HEAD rev-parse f8017f1e494ba998c439f9f3acb66771490de0a0 comprobado; status tuvo error de herramienta/permiso, no acredité limpieza Git propia.

## Contraste independiente

I01-I03: wire16+body65536, exact bodies y estados cerrados, chunks contiguos, Init/Begin idéntico, digest input y response con dominio definido, End/counts/EOF y receipts externos separan protocolo de cierre. El requisito canónico de validar raster length forma parte de esta revisión; implementador debe contrastar total recibido con stride*height, no aceptar sólo contigüidad/digest. Primer byte antes cierre observado y errores tras joins son sticky; Pending/cancel/stale nunca publica. R3UT se encapsula completo sin reinterpretar sus tablas.

I04: comprobadas fuentes físicas cpdf_page-7881.cpp, fpdf_transformpage-7881.cpp y fpdf_view-7881.cpp. Getter directo lee GetDict()->GetArrayFor; bbox efectivo deriva herencia/fallback/intersección. CPDF_Page::UpdateDimensions y GetDisplayMatrix soportan una matriz externa M que cumple M(N(p))=D(p); no duplicar rotación intrínseca. T13 corners concuerdan con las cuatro transformaciones algebraicas fijadas; matrices y clipping quedan sujetos a FFI y AE futuras. FS_RECTF left/top/right/bottom se mapea explícitamente a x0/y0/x1/y1; coordenadas negativas son válidas. No normalización ni outputs leídos tras false.

I05: header físico7881 BGRA4 declara componentes independientes de alpha; BGRA_Premul5 requiere Skia. fpdf_view-7881.cpp GetFormat discrimina kBgra/kBgraPremul. R5 elige explícitamente straight2, buffer externo único, fondo transparente, stride y GetFormat comprobados; peer no aplica desigualdad RGB<=A. Valores nativos/tolerancia se fijarán prospectivamente con fixture D2 antes outputs, no se derivan de un golden autor. Nueva API/whitelist y render no están aceptados runtime por esta lectura.

I06-I07: texto válido con mapping parcial permanece NativeDerivedUnverified; no convertir source FAIL en PASS ni reservar innerLength más tablas. Raster y text son requests separados. H40/D24, source1MiB/pages256/raster4MiB/body65536 siguen cuotas experimentales; inventario exacto y simultaneidad están OPEN_COMPILE D1, no una afirmación RAM. DeltaR5 fija stage/codes y reservado; input inválido/setup/early response permanecen HostError, ContentFailure requiere transacción íntegra y no publica artifact.

I08: T01-T16 y P01-P26 son prospectivos NOT_RUN y proporcionan mecanismo de fixtures/mutaciones/fronteras/cierre/geometry/alpha. Oráculos independientes adicionales I01-I08 siguen disponibles para preparación D2. No ejecuté PDFium, código productivo, fixture PDF nueva, ABI Rust ni UTF16; no usé outputs del autor como golden. R1-R4 y FAILs siguen privados/preservados. ActorR2 aceptado sólo como contrato; no acepta wire por asociación.

## Límites

Este dictamen no acredita D1/D2/D3, memoria compilada, containment/runtime Rust, pixels DLL7881, FFI nuevas, viewport/terminal, QA Windows o Unix ni cierre P1/RV204/P2-P5. No autoriza publicación, perfiles, bundle ni implementación paralela. No se modificó candidato o producción.
