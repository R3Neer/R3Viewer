# YAML R3 - aceptación independiente funcional

Dictamen: PASS sobre cobertura ejecutada; sin defectos funcionales pendientes en los casos de este informe. No acredita producto completo, QA visual o runtime Unix. R1 permanece NO ACEPTADO en REPORT-R1.md; R2 delta resolvió sus dos defectos; el cierre de este informe corresponde al snapshot R3 exacto.

## Identidad y procedencia

R3 MANIFEST SHA256 472C490C51152D80A6FF76393A46BFD2D4BCE3DC41ABD10AEDCB46E4093EFBF3; r3viewer.exe 29EBF22FF82B03FDBF039B46FBD086E766EB3F67BB4B0945E796F0E3C5F6B15A; companion E885DE7F826DA9520EA7E6A11139F57F966A9BDB4E9CFEB5653A6BFDB6D5567E. Hashes verificados en ejecución. No editar producción, snapshot, corpus ni oráculos; evidencia nueva exclusivamente TEMP/r3viewer-yaml-ae-resume.

Original EXPECTATIONS 343D6938DDE6A91A7EA26EAD84267F9AE030A4EB30695DC149A2D389AF18A84B conserva bytes. Correcciones exclusivamente categorías: suplemento custom SHA044C3ACC4B1760604D343422F15CFB502D03815375C95AA844238B07545D7CA6, y suplemento14 SHA EAAE7DD1F0F7D60D8F016442ED169770BC556F9CC74CD9EBEEE1BD040B19DE22 adjudicado independientemente SHA BACC4F7B61990868A9C30965741A4562BE6AA24B3A2E2C804BC359C23B4C6477. No cambiaron Ready/decoded/URI/raw/spans. bool en salida humana se normalizó a categoría Boolean sin alterar semántica. Lista exacta14 conservada en oracle-category-discrepancies.json.

## Corpus y spans

R1 ejecutó202CLI sobre101fixtures:82 preparación y19flow, fuente y estructura por separado, código0, fuente con bytes exactos, Ready/Unavailable completos. R3/inline-adjudicated.json evalúa resultados conservados con los15ajustes:101PASS de las comprobaciones declaradas, expandedURI/decoded exactos para scalars con esos campos. Reutilización justificada por hashes R1=R3 de yaml_worker, structure_model, structure_protocol, structure_output, structure_worker, móduloNu y companion; delta está en búsqueda/presentación. No se afirma igualdad byte de aplicación, que cambió.

Ejecución nueva contra fuenteR3 exacta: cargo test --locked --offline --manifest-path SNAPSHOT_R3/Cargo.toml --target-dir TEMP/r3viewer-yaml-review-resume/target --lib independent_ -- --nocapture.3 tests PASS: 82 estados originales+53 scalarspans,42 flowselections yprofundidad lógica y cancel. Spans:53 Some scalars y42flow (37 Some exactos +5 None prescritos). Selecciónflow deriva pathsordinales deloracle independiente congelado; compara SourceSpan yrawslice. Generators y assertions inspeccionados. Reutilizar arnés del autor con expectativas externas no es un oracle derivado de resultados del adapter. Limitación: casosalias scalar se localizan por span esperado y verifican raw/kind; no acreditan por sí solos identidadalias independiente. ConPTY alias item4 ycycle acreditan navegación/identidad por ruta aparte. Primer intento --bin ejecutó0tests; no cuenta como evidencia de gate. Logs spans-test.log/spans-test-lib.log preservan ambos.

## Producto R3 mediante ConPTY

12 sesiones interactivas Nu0.115.1 -n --plugins [companion del snapshot], con módulo snapshot, sin instalación ni perfil. Todos q0 y forced_cleanup false. Es funcional mediante emulación ANSI, no observación visual de terminales.

R3/domains-fresh.json:6 sesiones nuevas. /␛ devuelveitem 1, glifo; o conserva token "␛" completo; n/N no atraviesa ESC del item 0. /needle_uri devuelve URI del item 2; o usa scalar original "plain", no fabrica rango de propiedad del tag; Tab/n/N mantiene dominio URI. /double quoted, /anchor ID y /documento: sin coincidencias; /anchorZ identifica alias del item 4 mediante descriptor canónico sin expansión. Resuelve R1-YAE1 yR1-YAE2.

R3/extended.json:decoded ESC+glifo literal antes de needle: highlights6celdas exactas, n segundo y N al primero, resize65x18, o token quoted exacto, Tab restaura yBack devuelve hit previo. El chequeo deR2 del contador en título estrecho fallaba por truncado; R3 verifica pane esperado, no interpreta ese fallo del arnés como defecto.

Mismo extended:ciclo de alias135Enter alcanza128retornos, acciones extra y n se rechazan sin cambiar selección; Back baja a 127 y permite n hasta 128; Back devuelve descriptor del alias anterior;130 Back vacía cadena completa y vuelve raíz del stream con 0 retornos. No se invoca expansión de alias.

R3/performance.json:scalar de 200000'a', consulta a. Forward0.113s, N al último0.174s y N al previo0.175s con highlights correctos; tiempos de esta máquina/sesión, no garantía general. Evita certificación de complejidad algorítmica solo por este timing; contador independiente corresponde al adversario.

R3/original-flow.json:3 sesiones y4selecciones:[] vacío,{} vacío,clave contenedora[] vacía yvalor {} vacío; o abre Fuente y resalta exactamente2celdas de delimitadores. Complementa95rangos de adapter con interacción real desde current sin celda ydesdeclave contenedora.

R2/source-nu.json:3 fixtures del delta fuente con bytes exactos y código 0; NuString explícito produce estructura0; integer tipado yopen--raw BinaryByteStream rechazan con código 2. open--raw describe byte stream; no se etiquetó erróneamente como StringByteStream. Parser/output/plugin/módulo idénticos entre R2 y R3. No prueba aquí StringByteStream textual separado ni matriz Nu completa.

## Límites de cierre

No se repitieron lifecycle del Job Windows512MiB/deadlines/cancel con código 130/retry manual/reap, presupuestos máximos source 128 MiB, arena 64 MiB e items 1 M, ni todas las regresiones TOML/JSON/Nu. Este informe no sustituye esos gates previos ni los declara aprobados por procedencia. No visual de WT/WezTerm, no runtime Unix, no producto ni de RV-106 completo. Los 95 spans sí está verificada ahora y no queda pendiente en esta cobertura. Pruebas automáticas de workspace y adversario son evidencia separada del coordinador, no ejecución atribuida a este agente. Consulta memoria no dio hits relevantes de R3Viewer y no se usó para conclusiones.
